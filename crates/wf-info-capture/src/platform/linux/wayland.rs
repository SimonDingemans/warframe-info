use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    time::Duration,
};

use ashpd::desktop::{
    screencast::{
        CursorMode, Screencast, SelectSourcesOptions, SourceType, Stream as ScreencastStream,
        Streams,
    },
    PersistMode, Session,
};
use image::{DynamicImage, RgbaImage};
use pipewire as pw;
use pw::{properties::properties, spa};

use crate::{
    CaptureError, CaptureFuture, CapturePermissionFuture, CaptureResult, ScreenCapture,
    ScreenCaptureSource, Screenshot,
};

#[derive(Debug, Clone, Default)]
pub struct LinuxWaylandCapture;

impl LinuxWaylandCapture {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenCapture for LinuxWaylandCapture {
    fn capture_screen(&self) -> CaptureFuture<'_> {
        Box::pin(async { capture_screen_with_portal().await })
    }

    fn request_permission(&self) -> CapturePermissionFuture<'_> {
        Box::pin(async { request_screen_capture_permission_with_portal().await })
    }
}

pub fn reset_screencast_token() -> CaptureResult<()> {
    remove_screencast_token(&screencast_token_path())
}

async fn capture_screen_with_portal() -> CaptureResult<Screenshot> {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(CaptureError::NotWaylandSession);
    }

    let (stream, fd) = open_screen_stream().await?;
    let source = screen_capture_source(&stream);
    let image = capture_pipewire_frame(stream.pipe_wire_node_id(), fd)?;

    Ok(Screenshot { image, source })
}

async fn request_screen_capture_permission_with_portal() -> CaptureResult<()> {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(CaptureError::NotWaylandSession);
    }

    let (_session, response) = request_screen_capture_response().await?;

    if response.streams().is_empty() {
        return Err(CaptureError::WaylandScreencastMissingStream);
    }

    Ok(())
}

async fn open_screen_stream() -> CaptureResult<(ScreencastStream, std::os::fd::OwnedFd)> {
    let proxy = Screencast::new().await?;
    let (session, response) = request_screen_capture_response_with_proxy(&proxy).await?;

    let stream = response
        .streams()
        .first()
        .cloned()
        .ok_or(CaptureError::WaylandScreencastMissingStream)?;
    let fd = proxy
        .open_pipe_wire_remote(&session, Default::default())
        .await?;

    Ok((stream, fd))
}

async fn request_screen_capture_response() -> CaptureResult<(Session<Screencast>, Streams)> {
    let proxy = Screencast::new().await?;
    request_screen_capture_response_with_proxy(&proxy).await
}

async fn request_screen_capture_response_with_proxy(
    proxy: &Screencast,
) -> CaptureResult<(Session<Screencast>, Streams)> {
    let available_sources = proxy.available_source_types().await?;
    if !available_sources.contains(SourceType::Monitor) {
        return Err(CaptureError::WaylandScreenCaptureUnavailable);
    }

    let token_path = screencast_token_path();
    let restore_token = read_screencast_token(&token_path)?;
    let session = proxy.create_session(Default::default()).await?;

    proxy
        .select_sources(
            &session,
            SelectSourcesOptions::default()
                .set_cursor_mode(CursorMode::Hidden)
                .set_sources(Some(SourceType::Monitor.into()))
                .set_multiple(false)
                .set_restore_token(restore_token.as_deref())
                .set_persist_mode(PersistMode::ExplicitlyRevoked),
        )
        .await?;

    let response = proxy
        .start(&session, None, Default::default())
        .await?
        .response()?;

    if let Some(token) = response.restore_token() {
        write_screencast_token(&token_path, token)?;
    }

    Ok((session, response))
}

fn screen_capture_source(stream: &ScreencastStream) -> Option<ScreenCaptureSource> {
    let (width, height) = stream.size()?;
    if width <= 0 || height <= 0 {
        return None;
    }

    Some(ScreenCaptureSource {
        size: (width as u32, height as u32),
    })
}

struct PipeWireFrameState {
    format: spa::param::video::VideoInfoRaw,
    sender: Sender<CaptureResult<DynamicImage>>,
}

fn capture_pipewire_frame(node_id: u32, fd: std::os::fd::OwnedFd) -> CaptureResult<DynamicImage> {
    pw::init();

    let mainloop = pw::main_loop::MainLoopRc::new(None).map_err(pipewire_error)?;
    let context = pw::context::ContextBox::new(mainloop.loop_(), None).map_err(pipewire_error)?;
    let core = context.connect_fd(fd, None).map_err(pipewire_error)?;
    let stream = pw::stream::StreamBox::new(
        &core,
        "wf-info-capture",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )
    .map_err(pipewire_error)?;
    let (sender, receiver) = mpsc::channel();
    let process_mainloop = mainloop.clone();
    let timeout_mainloop = mainloop.clone();
    let timeout_sender = sender.clone();

    let state = PipeWireFrameState {
        format: Default::default(),
        sender,
    };

    let _timeout = mainloop.loop_().add_timer(move |_| {
        let _ = timeout_sender.send(Err(CaptureError::InvalidPipeWireFrame));
        timeout_mainloop.quit();
    });
    _timeout
        .update_timer(Some(Duration::from_secs(5)), None)
        .into_result()
        .map_err(pipewire_error)?;

    let _listener = stream
        .add_local_listener_with_user_data(state)
        .param_changed(|_, state, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != pw::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let Ok((media_type, media_subtype)) = pw::spa::param::format_utils::parse_format(param)
            else {
                return;
            };
            if media_type != pw::spa::param::format::MediaType::Video
                || media_subtype != pw::spa::param::format::MediaSubtype::Raw
            {
                return;
            }

            let _ = state.format.parse(param);
        })
        .process(move |stream, state| {
            let result = match stream.dequeue_buffer() {
                Some(mut buffer) => pipewire_buffer_to_image(&mut buffer, state.format),
                None => Err(CaptureError::InvalidPipeWireFrame),
            };
            let _ = state.sender.send(result);
            process_mainloop.quit();
        })
        .register()
        .map_err(pipewire_error)?;

    let param_bytes = pipewire_video_format_param_bytes()?;
    let param =
        spa::pod::Pod::from_bytes(&param_bytes).ok_or(CaptureError::InvalidPipeWireFrame)?;
    let mut params = [param];
    stream
        .connect(
            spa::utils::Direction::Input,
            Some(node_id),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .map_err(pipewire_error)?;

    mainloop.run();
    receiver
        .recv_timeout(Duration::from_secs(1))
        .map_err(|_| CaptureError::InvalidPipeWireFrame)?
}

fn pipewire_video_format_param_bytes() -> CaptureResult<Vec<u8>> {
    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaType,
            Id,
            pw::spa::param::format::MediaType::Video
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::param::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRA,
            pw::spa::param::video::VideoFormat::BGRx,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::BGR,
        )
    );
    let bytes = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .map_err(pipewire_error)?
    .0
    .into_inner();

    Ok(bytes)
}

fn pipewire_buffer_to_image(
    buffer: &mut pw::buffer::Buffer,
    format: spa::param::video::VideoInfoRaw,
) -> CaptureResult<DynamicImage> {
    let datas = buffer.datas_mut();
    let Some(data) = datas.first_mut() else {
        return Err(CaptureError::InvalidPipeWireFrame);
    };

    let offset = data.chunk().offset() as usize;
    let size = data.chunk().size() as usize;
    let stride = data.chunk().stride();
    let Some(bytes) = data.data() else {
        return Err(CaptureError::InvalidPipeWireFrame);
    };
    let frame_end = offset
        .checked_add(size)
        .ok_or(CaptureError::InvalidPipeWireFrame)?;
    let frame = bytes
        .get(offset..frame_end)
        .ok_or(CaptureError::InvalidPipeWireFrame)?;

    rgba_image_from_pipewire_frame(frame, stride, format).map(DynamicImage::ImageRgba8)
}

fn rgba_image_from_pipewire_frame(
    frame: &[u8],
    stride: i32,
    format: spa::param::video::VideoInfoRaw,
) -> CaptureResult<RgbaImage> {
    let size = format.size();
    let width = size.width;
    let height = size.height;
    if width == 0 || height == 0 || stride == 0 {
        return Err(CaptureError::InvalidPipeWireFrame);
    }

    let format = format.format();
    let width = width as usize;
    let height = height as usize;
    let stride = stride.unsigned_abs() as usize;
    let bytes_per_pixel = match format {
        spa::param::video::VideoFormat::RGB | spa::param::video::VideoFormat::BGR => 3,
        spa::param::video::VideoFormat::RGBA
        | spa::param::video::VideoFormat::RGBx
        | spa::param::video::VideoFormat::BGRA
        | spa::param::video::VideoFormat::BGRx => 4,
        _ => {
            return Err(CaptureError::UnsupportedPipeWireFormat {
                format: format!("{format:?}"),
            });
        }
    };

    if stride < width * bytes_per_pixel || frame.len() < stride * height {
        return Err(CaptureError::InvalidPipeWireFrame);
    }

    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in 0..height {
        let row = &frame[row * stride..row * stride + width * bytes_per_pixel];
        for pixel in row.chunks_exact(bytes_per_pixel) {
            match format {
                spa::param::video::VideoFormat::RGB => {
                    rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
                }
                spa::param::video::VideoFormat::BGR => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
                }
                spa::param::video::VideoFormat::RGBA => {
                    rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], pixel[3]]);
                }
                spa::param::video::VideoFormat::RGBx => {
                    rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
                }
                spa::param::video::VideoFormat::BGRA => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
                spa::param::video::VideoFormat::BGRx => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
                }
                _ => unreachable!(),
            }
        }
    }

    RgbaImage::from_raw(width as u32, height as u32, rgba).ok_or(CaptureError::InvalidPipeWireFrame)
}

fn screencast_token_path() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("warframe-info")
        .join("wayland-monitor-screencast-token")
}

fn read_screencast_token(path: &PathBuf) -> CaptureResult<Option<String>> {
    match fs::read_to_string(path) {
        Ok(token) => {
            let token = token.trim().to_string();
            Ok((!token.is_empty()).then_some(token))
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CaptureError::WaylandScreencastToken {
            path: path.clone(),
            source,
        }),
    }
}

fn write_screencast_token(path: &PathBuf, token: &str) -> CaptureResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CaptureError::WaylandScreencastToken {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, token).map_err(|source| CaptureError::WaylandScreencastToken {
        path: path.clone(),
        source,
    })
}

fn remove_screencast_token(path: &PathBuf) -> CaptureResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(CaptureError::WaylandScreencastToken {
            path: path.clone(),
            source,
        }),
    }
}

fn pipewire_error(error: impl std::fmt::Display) -> CaptureError {
    CaptureError::PipeWire {
        message: error.to_string(),
    }
}

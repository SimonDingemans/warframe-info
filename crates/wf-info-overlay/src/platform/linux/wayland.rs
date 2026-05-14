use ashpd::desktop::{
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType, StartCastOptions},
    PersistMode,
};
use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1, zxdg_output_v1::ZxdgOutputV1,
};

use crate::DisplayOutput;

#[derive(Debug, Clone, Default)]
pub struct LinuxWaylandDisplayBackend;

impl LinuxWaylandDisplayBackend {
    pub fn new() -> Self {
        Self
    }
}

impl crate::DisplayBackend for LinuxWaylandDisplayBackend {
    fn display_outputs(&self) -> crate::DisplayOutputsFuture<'_> {
        Box::pin(async { display_outputs().await })
    }
}

async fn display_outputs() -> crate::DisplayResult<Vec<DisplayOutput>> {
    let proxy = Screencast::new().await.map_err(|error| error.to_string())?;
    let session = proxy
        .create_session(Default::default())
        .await
        .map_err(|error| error.to_string())?;

    proxy
        .select_sources(
            &session,
            SelectSourcesOptions::default()
                .set_cursor_mode(CursorMode::Hidden)
                .set_sources(Some(SourceType::Monitor.into()))
                .set_multiple(true)
                .set_persist_mode(PersistMode::DoNot),
        )
        .await
        .map_err(|error| error.to_string())?;

    let response = proxy
        .start(&session, None, StartCastOptions::default())
        .await
        .map_err(|error| error.to_string())?
        .response()
        .map_err(|error| error.to_string())?;

    let wayland_outputs = detect_wayland_outputs().unwrap_or_default();

    Ok(response
        .streams()
        .iter()
        .filter_map(|stream| {
            let position = stream.position()?;
            let (width, height) = stream.size()?;
            if width <= 0 || height <= 0 {
                return None;
            }

            let name = find_matching_wayland_output(&wayland_outputs, position, (width, height))
                .map(|output| output.name.clone())
                .or_else(|| {
                    stream
                        .mapping_id()
                        .or_else(|| stream.id())
                        .map(ToOwned::to_owned)
                });

            Some(DisplayOutput {
                name,
                position,
                size: (width as u32, height as u32),
            })
        })
        .collect())
}

fn find_matching_wayland_output(
    outputs: &[WaylandOutputInfo],
    position: (i32, i32),
    size: (i32, i32),
) -> Option<&WaylandOutputInfo> {
    outputs
        .iter()
        .filter(|output| close_pair(output.position, position) && close_pair(output.size, size))
        .min_by_key(|output| {
            abs_diff(output.position.0, position.0)
                + abs_diff(output.position.1, position.1)
                + abs_diff(output.size.0, size.0)
                + abs_diff(output.size.1, size.1)
        })
}

fn close_pair(left: (i32, i32), right: (i32, i32)) -> bool {
    const ROUNDING_TOLERANCE: u32 = 2;

    abs_diff(left.0, right.0) <= ROUNDING_TOLERANCE
        && abs_diff(left.1, right.1) <= ROUNDING_TOLERANCE
}

fn abs_diff(left: i32, right: i32) -> u32 {
    left.abs_diff(right)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaylandOutputInfo {
    name: String,
    position: (i32, i32),
    size: (i32, i32),
}

#[derive(Default)]
struct WaylandOutputState {
    outputs: Vec<(u32, wl_output::WlOutput)>,
    xdg_outputs: Vec<WaylandOutputInfoState>,
}

impl WaylandOutputState {
    fn track_output(
        &mut self,
        name: u32,
        output: wl_output::WlOutput,
        manager: &ZxdgOutputManagerV1,
        queue_handle: &QueueHandle<Self>,
    ) {
        self.outputs.push((name, output.clone()));
        self.xdg_outputs
            .push(WaylandOutputInfoState::new(manager.get_xdg_output(
                &output,
                queue_handle,
                (),
            )));
    }

    fn apply_xdg_output_event(&mut self, proxy: &ZxdgOutputV1, event: zxdg_output_v1::Event) {
        let Some(output) = self
            .xdg_outputs
            .iter_mut()
            .find(|output| output.matches_proxy(proxy))
        else {
            return;
        };

        output.apply_event(event);
    }

    fn into_detected_outputs(self) -> Vec<WaylandOutputInfo> {
        self.xdg_outputs
            .into_iter()
            .filter_map(WaylandOutputInfoState::into_detected_output)
            .collect()
    }
}

struct WaylandOutputInfoState {
    xdg_output: ZxdgOutputV1,
    details: WaylandOutputDetails,
}

impl WaylandOutputInfoState {
    fn new(xdg_output: ZxdgOutputV1) -> Self {
        Self {
            xdg_output,
            details: WaylandOutputDetails::default(),
        }
    }

    fn matches_proxy(&self, proxy: &ZxdgOutputV1) -> bool {
        self.xdg_output == *proxy
    }

    fn apply_event(&mut self, event: zxdg_output_v1::Event) {
        self.details.apply_event(event);
    }

    fn into_detected_output(self) -> Option<WaylandOutputInfo> {
        self.details.into_detected_output()
    }
}

#[derive(Default)]
struct WaylandOutputDetails {
    name: Option<String>,
    position: Option<(i32, i32)>,
    size: Option<(i32, i32)>,
}

impl WaylandOutputDetails {
    fn apply_event(&mut self, event: zxdg_output_v1::Event) {
        match event {
            zxdg_output_v1::Event::Name { name } => self.name = Some(name),
            zxdg_output_v1::Event::LogicalPosition { x, y } => self.position = Some((x, y)),
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                self.size = Some((width, height));
            }
            _ => {}
        }
    }

    fn into_detected_output(self) -> Option<WaylandOutputInfo> {
        Some(WaylandOutputInfo {
            name: self.name?,
            position: self.position?,
            size: self.size?,
        })
    }
}

fn detect_wayland_outputs() -> Result<Vec<WaylandOutputInfo>, String> {
    let connection = Connection::connect_to_env().map_err(|error| error.to_string())?;
    let (globals, mut event_queue) = registry_queue_init::<WaylandOutputState>(&connection)
        .map_err(|error| error.to_string())?;
    let queue_handle = event_queue.handle();
    let manager = globals
        .bind::<ZxdgOutputManagerV1, _, _>(&queue_handle, 1..=3, ())
        .map_err(|error| error.to_string())?;

    let output_globals = globals.contents().with_list(|globals| {
        globals
            .iter()
            .filter(|global| global.interface == wl_output::WlOutput::interface().name)
            .map(|global| (global.name, global.version))
            .collect::<Vec<_>>()
    });

    let mut state = WaylandOutputState::default();
    for (name, version) in output_globals {
        let output = globals.registry().bind::<wl_output::WlOutput, _, _>(
            name,
            version.min(4),
            &queue_handle,
            (),
        );
        state.track_output(name, output, &manager, &queue_handle);
    }

    event_queue
        .roundtrip(&mut state)
        .map_err(|error| error.to_string())?;

    Ok(state.into_detected_outputs())
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandOutputState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZxdgOutputV1, ()> for WaylandOutputState {
    fn event(
        state: &mut Self,
        proxy: &ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        state.apply_xdg_output_event(proxy, event);
    }
}

delegate_noop!(WaylandOutputState: ignore wl_output::WlOutput);
delegate_noop!(WaylandOutputState: ignore ZxdgOutputManagerV1);

pub mod platform;

use std::{future::Future, path::PathBuf, pin::Pin};

use image::DynamicImage;
use thiserror::Error;

pub type CaptureResult<T> = Result<T, CaptureError>;
pub type DynScreenCapture = Box<dyn ScreenCapture>;
pub type CaptureFuture<'a> = Pin<Box<dyn Future<Output = CaptureResult<Screenshot>> + Send + 'a>>;
pub type CapturePermissionFuture<'a> = Pin<Box<dyn Future<Output = CaptureResult<()>> + Send + 'a>>;

pub trait ScreenCapture: Send + Sync {
    fn capture_screen(&self) -> CaptureFuture<'_>;

    fn request_permission(&self) -> CapturePermissionFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub image: DynamicImage,
    pub source: Option<ScreenCaptureSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCaptureSource {
    pub size: (u32, u32),
}

pub async fn capture_screen() -> CaptureResult<Screenshot> {
    platform::default_capture().capture_screen().await
}

pub async fn request_screen_capture_permission() -> CaptureResult<()> {
    platform::default_capture().request_permission().await
}

pub fn reset_screen_capture_restore_token() -> CaptureResult<()> {
    platform::reset_screen_capture_restore_token()
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("screen capture is only implemented for Linux Wayland")]
    UnsupportedPlatform,

    #[error("Linux Wayland capture requires WAYLAND_DISPLAY to be set")]
    NotWaylandSession,

    #[error("portal request failed")]
    Portal(#[from] ashpd::Error),

    #[error("Wayland screencast portal does not offer screen capture")]
    WaylandScreenCaptureUnavailable,

    #[error("Wayland screencast portal did not return a PipeWire stream")]
    WaylandScreencastMissingStream,

    #[error("failed to access Wayland screencast restore token at {}", path.display())]
    WaylandScreencastToken {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("PipeWire screencast failed: {message}")]
    PipeWire { message: String },

    #[error("PipeWire returned an unsupported video format: {format}")]
    UnsupportedPipeWireFormat { format: String },

    #[error("PipeWire returned an invalid video frame")]
    InvalidPipeWireFrame,
}

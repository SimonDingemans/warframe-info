pub mod platform;

use std::{future::Future, path::PathBuf, pin::Pin};

use image::DynamicImage;
use thiserror::Error;

pub type CaptureResult<T> = Result<T, CaptureError>;
pub type DynScreenCapture = Box<dyn ScreenCapture>;
pub type CaptureFuture<'a> = Pin<Box<dyn Future<Output = CaptureResult<Screenshot>> + Send + 'a>>;

pub trait ScreenCapture: Send + Sync {
    fn capture_screen(&self) -> CaptureFuture<'_>;
}

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub image: DynamicImage,
}

pub async fn capture_screen() -> CaptureResult<Screenshot> {
    platform::default_capture().capture_screen().await
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("screen capture is only implemented for Linux Wayland")]
    UnsupportedPlatform,

    #[error("Linux Wayland capture requires WAYLAND_DISPLAY to be set")]
    NotWaylandSession,

    #[error("screenshot portal request failed")]
    Portal(#[from] ashpd::Error),

    #[error("screenshot portal returned an unsupported URI: {uri}")]
    UnsupportedScreenshotUri { uri: String },

    #[error("screenshot portal returned an invalid file URI: {uri}")]
    InvalidScreenshotUri { uri: String },

    #[error("failed to open screenshot image at {}", path.display())]
    OpenImage {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
}

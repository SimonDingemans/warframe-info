use std::{future::Future, path::PathBuf, pin::Pin};

use image::DynamicImage;
use thiserror::Error;

pub type CaptureResult<T> = Result<T, CaptureError>;
pub type DynScreenCapture = Box<dyn ScreenCapture>;
pub type CaptureFuture<'a> = Pin<Box<dyn Future<Output = CaptureResult<Screenshot>> + Send + 'a>>;
pub type CapturePermissionFuture<'a> = Pin<Box<dyn Future<Output = CaptureResult<()>> + Send + 'a>>;

pub trait ScreenCapture: Send + Sync {
    fn capabilities(&self) -> CaptureCapabilities {
        CaptureCapabilities::default()
    }

    fn capture_screen(&self) -> CaptureFuture<'_>;

    fn request_permission(&self) -> CapturePermissionFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    fn reset_permission_state(&self) -> CaptureResult<()> {
        Ok(())
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaptureCapabilities {
    pub permission_request: bool,
    pub permission_reset: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UnsupportedCapture;

impl ScreenCapture for UnsupportedCapture {
    fn capture_screen(&self) -> CaptureFuture<'_> {
        Box::pin(async { Err(CaptureError::UnsupportedBackend) })
    }
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("screen capture is not supported by the selected backend")]
    UnsupportedBackend,

    #[error("{message}")]
    SessionUnavailable { message: String },

    #[error("{message}")]
    SourceUnavailable { message: String },

    #[error("{message}")]
    RequestFailed { message: String },

    #[error("failed to access screen capture restore token at {}", path.display())]
    RestoreToken {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{message}")]
    FrameCaptureFailed { message: String },

    #[error("screen capture returned an unsupported video format: {format}")]
    UnsupportedFrameFormat { format: String },

    #[error("screen capture returned an invalid video frame")]
    InvalidFrame,

    #[error("{backend} backend failed: {message}")]
    Backend {
        backend: &'static str,
        message: String,
    },
}

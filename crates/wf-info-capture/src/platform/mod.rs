#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub fn default_capture() -> crate::DynScreenCapture {
    linux::default_capture()
}

#[cfg(not(target_os = "linux"))]
pub fn default_capture() -> crate::DynScreenCapture {
    Box::new(UnsupportedCapture)
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone, Default)]
struct UnsupportedCapture;

#[cfg(not(target_os = "linux"))]
impl crate::ScreenCapture for UnsupportedCapture {
    fn capture_screen(&self) -> crate::CaptureFuture<'_> {
        Box::pin(async { Err(crate::CaptureError::UnsupportedPlatform) })
    }
}

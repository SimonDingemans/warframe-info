pub mod wayland;

pub fn default_capture() -> crate::DynScreenCapture {
    Box::new(wayland::LinuxWaylandCapture::new())
}

pub fn reset_screen_capture_restore_token() -> crate::CaptureResult<()> {
    wayland::reset_screencast_token()
}

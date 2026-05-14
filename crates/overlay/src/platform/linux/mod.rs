pub mod wayland;

pub fn default_display_backend() -> crate::DynDisplayBackend {
    Box::new(wayland::LinuxWaylandDisplayBackend::new())
}

pub fn reset_display_restore_token() -> crate::DisplayResult<()> {
    wayland::reset_screencast_token()
}

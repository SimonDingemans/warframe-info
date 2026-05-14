pub mod wayland;

pub fn default_display_backend() -> crate::DynDisplayBackend {
    Box::new(wayland::LinuxWaylandDisplayBackend::new())
}

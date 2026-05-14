pub mod wayland;

pub fn default_capture() -> crate::DynScreenCapture {
    Box::new(wayland::LinuxWaylandCapture::new())
}

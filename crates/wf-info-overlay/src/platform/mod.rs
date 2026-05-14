#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

#[cfg(target_os = "linux")]
pub fn default_display_backend() -> crate::DynDisplayBackend {
    linux::default_display_backend()
}

#[cfg(target_os = "linux")]
pub fn reset_display_restore_token() -> crate::DisplayResult<()> {
    linux::reset_display_restore_token()
}

#[cfg(not(target_os = "linux"))]
pub fn default_display_backend() -> crate::DynDisplayBackend {
    Box::new(unsupported::UnsupportedDisplayBackend)
}

#[cfg(not(target_os = "linux"))]
pub fn reset_display_restore_token() -> crate::DisplayResult<()> {
    unsupported::reset_display_restore_token()
}

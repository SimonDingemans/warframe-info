#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
#[cfg(not(target_os = "linux"))]
#[path = "unsupported.rs"]
mod imp;

use super::SystemShortcutIntegration;

pub(super) fn system_shortcuts() -> &'static dyn SystemShortcutIntegration {
    &imp::SYSTEM_SHORTCUTS
}

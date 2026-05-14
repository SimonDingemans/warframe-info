use super::{GlobalShortcutBackend, Platform};

pub(super) static PLATFORM: UnsupportedPlatform = UnsupportedPlatform;

pub(super) struct UnsupportedPlatform;

impl Platform for UnsupportedPlatform {
    fn global_shortcut_backend(&self) -> GlobalShortcutBackend {
        GlobalShortcutBackend::Native
    }
}

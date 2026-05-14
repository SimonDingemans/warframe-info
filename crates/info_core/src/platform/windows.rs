use super::{GlobalShortcutBackend, Platform};

pub(super) static PLATFORM: WindowsPlatform = WindowsPlatform;

pub(super) struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn global_shortcut_backend(&self) -> GlobalShortcutBackend {
        GlobalShortcutBackend::Native
    }
}

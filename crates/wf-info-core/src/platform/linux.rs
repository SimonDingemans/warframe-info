use std::env;

use super::{GlobalShortcutBackend, Platform};

pub(super) static PLATFORM: LinuxPlatform = LinuxPlatform;

pub(super) struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn global_shortcut_backend(&self) -> GlobalShortcutBackend {
        if env::var_os("WAYLAND_DISPLAY").is_some() {
            GlobalShortcutBackend::SystemConfigured
        } else {
            GlobalShortcutBackend::Native
        }
    }
}

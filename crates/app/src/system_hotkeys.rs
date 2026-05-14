use hotkeys::{HotkeyBackend, ShortcutIntegrationHandle};
use info_core::{platform::GlobalShortcutBackend, AppSettings};

pub(crate) fn hotkey_backend() -> HotkeyBackend {
    if has_system_shortcut_configuration() {
        HotkeyBackend::Integrated(system_shortcut_integration())
    } else {
        HotkeyBackend::Native
    }
}

pub(crate) fn has_system_shortcut_configuration() -> bool {
    info_core::platform::current().global_shortcut_backend()
        == GlobalShortcutBackend::SystemConfigured
}

pub(crate) fn configure_system_shortcuts(
    settings: AppSettings,
) -> hotkeys::BoxFuture<Result<String, String>> {
    system_shortcut_integration().configure_shortcuts(settings)
}

#[cfg(target_os = "linux")]
fn system_shortcut_integration() -> ShortcutIntegrationHandle {
    hotkeys_wayland::shortcut_integration()
}

#[cfg(not(target_os = "linux"))]
fn system_shortcut_integration() -> ShortcutIntegrationHandle {
    hotkeys::unsupported::shortcut_integration()
}

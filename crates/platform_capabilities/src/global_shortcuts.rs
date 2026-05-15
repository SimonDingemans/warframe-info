use hotkeys::{HotkeyBackend, ShortcutIntegrationHandle};
use info_core::AppSettings;

pub fn backend() -> HotkeyBackend {
    super::imp::hotkey_backend()
}

pub fn has_system_configuration() -> bool {
    system_integration().capabilities().system_configuration
}

pub fn configure_system_shortcuts(
    settings: AppSettings,
) -> hotkeys::BoxFuture<Result<String, String>> {
    system_integration().configure_shortcuts(settings)
}

fn system_integration() -> ShortcutIntegrationHandle {
    super::imp::system_shortcut_integration()
}

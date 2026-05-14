use hotkeys::HotkeyBackend;
use info_core::AppSettings;

pub(crate) fn hotkey_backend() -> HotkeyBackend {
    platform_capabilities::global_shortcuts::backend()
}

pub(crate) fn has_system_shortcut_configuration() -> bool {
    platform_capabilities::global_shortcuts::has_system_configuration()
}

pub(crate) fn configure_system_shortcuts(
    settings: AppSettings,
) -> hotkeys::BoxFuture<Result<String, String>> {
    platform_capabilities::global_shortcuts::configure_system_shortcuts(settings)
}

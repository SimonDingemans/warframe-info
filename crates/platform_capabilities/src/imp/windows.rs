pub(crate) fn hotkey_backend() -> hotkeys::HotkeyBackend {
    hotkeys::HotkeyBackend::Native
}

pub(crate) fn system_shortcut_integration() -> hotkeys::ShortcutIntegrationHandle {
    hotkeys::unsupported::shortcut_integration()
}

pub(crate) fn screen_capture_backend() -> capture::DynScreenCapture {
    Box::new(capture_windows::WindowsCapture::new())
}

pub(crate) async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
    Err(unsupported_overlay_message())
}

pub(crate) fn reset_display_restore_token() -> Result<(), String> {
    Ok(())
}

pub(crate) fn run_reward_overlay(_overlay: overlay::RewardOverlay) -> Result<(), String> {
    Err(unsupported_overlay_message())
}

fn unsupported_overlay_message() -> String {
    "Reward overlays are not supported on this platform".to_owned()
}

pub(crate) fn hotkey_backend() -> hotkeys::HotkeyBackend {
    if is_wayland_session() {
        hotkeys::HotkeyBackend::Integrated(hotkeys_wayland::shortcut_integration())
    } else {
        hotkeys::HotkeyBackend::Integrated(hotkeys::unsupported::shortcut_integration())
    }
}

pub(crate) fn system_shortcut_integration() -> hotkeys::ShortcutIntegrationHandle {
    if is_wayland_session() {
        hotkeys_wayland::shortcut_integration()
    } else {
        hotkeys::unsupported::shortcut_integration()
    }
}

pub(crate) fn screen_capture_backend() -> capture::DynScreenCapture {
    if is_wayland_session() {
        Box::new(capture_wayland::WaylandCapture::new())
    } else {
        Box::new(capture::UnsupportedCapture)
    }
}

pub(crate) async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
    if is_wayland_session() {
        overlay_wayland::display_outputs().await
    } else {
        Err(unsupported_native_linux_message())
    }
}

pub(crate) fn reset_display_restore_token() -> Result<(), String> {
    if is_wayland_session() {
        overlay_wayland::reset_display_restore_token()
    } else {
        Ok(())
    }
}

pub(crate) fn run_reward_overlay(overlay: overlay::RewardOverlay) -> Result<(), String> {
    if is_wayland_session() {
        overlay_wayland::run(overlay).map_err(|error| error.to_string())
    } else {
        Err(unsupported_native_linux_message())
    }
}

fn is_wayland_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn unsupported_native_linux_message() -> String {
    "Wayland is required; X11/native Linux capture and overlays are not supported yet".to_owned()
}

pub mod global_shortcuts {
    use hotkeys::{HotkeyBackend, ShortcutIntegrationHandle};
    use info_core::AppSettings;

    pub fn backend() -> HotkeyBackend {
        let integration = system_integration();

        if integration.capabilities().system_configuration {
            HotkeyBackend::Integrated(integration)
        } else {
            HotkeyBackend::Native
        }
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
}

pub mod screen_capture {
    pub fn backend() -> capture::DynScreenCapture {
        super::imp::screen_capture_backend()
    }
}

pub mod reward_overlay {
    pub async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
        super::imp::display_outputs().await
    }

    pub fn reset_display_restore_token() -> Result<(), String> {
        super::imp::reset_display_restore_token()
    }

    pub fn run(overlay: overlay::RewardOverlay) -> Result<(), String> {
        super::imp::run_reward_overlay(overlay)
    }
}

#[cfg(target_os = "linux")]
mod imp {
    pub(super) fn system_shortcut_integration() -> hotkeys::ShortcutIntegrationHandle {
        hotkeys_wayland::shortcut_integration()
    }

    pub(super) fn screen_capture_backend() -> capture::DynScreenCapture {
        Box::new(capture_wayland::WaylandCapture::new())
    }

    pub(super) async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
        overlay_wayland::display_outputs().await
    }

    pub(super) fn reset_display_restore_token() -> Result<(), String> {
        overlay_wayland::reset_display_restore_token()
    }

    pub(super) fn run_reward_overlay(overlay: overlay::RewardOverlay) -> Result<(), String> {
        overlay_wayland::run(overlay).map_err(|error| error.to_string())
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    pub(super) fn system_shortcut_integration() -> hotkeys::ShortcutIntegrationHandle {
        hotkeys::unsupported::shortcut_integration()
    }

    pub(super) fn screen_capture_backend() -> capture::DynScreenCapture {
        Box::new(capture::UnsupportedCapture)
    }

    pub(super) async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
        Err(unsupported_overlay_message())
    }

    pub(super) fn reset_display_restore_token() -> Result<(), String> {
        Ok(())
    }

    pub(super) fn run_reward_overlay(_overlay: overlay::RewardOverlay) -> Result<(), String> {
        Err(unsupported_overlay_message())
    }

    fn unsupported_overlay_message() -> String {
        "Reward overlays are not supported on this platform".to_owned()
    }
}

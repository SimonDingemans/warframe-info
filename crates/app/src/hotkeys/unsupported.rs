use iced::futures::{channel::mpsc, future, SinkExt};
use info_core::AppSettings;

use super::super::{BoxFuture, HotkeyEvent, SystemShortcutIntegration};

pub(super) static SYSTEM_SHORTCUTS: UnsupportedSystemShortcuts = UnsupportedSystemShortcuts;

pub(super) struct UnsupportedSystemShortcuts;

impl SystemShortcutIntegration for UnsupportedSystemShortcuts {
    fn registration_status(&self, _settings: &AppSettings) -> String {
        "System shortcut configuration is unavailable on this platform".to_owned()
    }

    fn configure_shortcuts(&self, settings: AppSettings) -> BoxFuture<Result<String, String>> {
        Box::pin(configure_system_shortcuts(settings))
    }

    fn watch_shortcuts(
        &self,
        _settings: AppSettings,
        sender: mpsc::Sender<HotkeyEvent>,
    ) -> BoxFuture<()> {
        Box::pin(watch_system_shortcuts(sender))
    }
}

async fn configure_system_shortcuts(_settings: AppSettings) -> Result<String, String> {
    Err("System shortcut configuration is unavailable on this platform".to_owned())
}

async fn watch_system_shortcuts(mut sender: mpsc::Sender<HotkeyEvent>) {
    let _ = sender
        .send(HotkeyEvent::Status(
            "System shortcut configuration is unavailable on this platform".to_owned(),
        ))
        .await;
    future::pending::<()>().await;
}

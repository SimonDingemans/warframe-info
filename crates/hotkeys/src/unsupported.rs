use futures::{channel::mpsc, future, SinkExt};
use info_core::{AppSettings, HotkeyEvent};

use crate::{BoxFuture, ShortcutIntegration, ShortcutIntegrationHandle};

static SHORTCUT_INTEGRATION: UnsupportedShortcutIntegration = UnsupportedShortcutIntegration;

pub fn shortcut_integration() -> ShortcutIntegrationHandle {
    ShortcutIntegrationHandle::new("unsupported", &SHORTCUT_INTEGRATION)
}

struct UnsupportedShortcutIntegration;

impl ShortcutIntegration for UnsupportedShortcutIntegration {
    fn registration_status(&self, _settings: &AppSettings) -> String {
        "System shortcut configuration is unavailable on this platform".to_owned()
    }

    fn configure_shortcuts(&self, _settings: AppSettings) -> BoxFuture<Result<String, String>> {
        Box::pin(async {
            Err("System shortcut configuration is unavailable on this platform".to_owned())
        })
    }

    fn watch_shortcuts(
        &self,
        _settings: AppSettings,
        sender: mpsc::Sender<HotkeyEvent>,
    ) -> BoxFuture<()> {
        Box::pin(watch_shortcuts(sender))
    }
}

async fn watch_shortcuts(mut sender: mpsc::Sender<HotkeyEvent>) {
    let _ = sender
        .send(HotkeyEvent::Status(
            "System shortcut configuration is unavailable on this platform".to_owned(),
        ))
        .await;
    future::pending::<()>().await;
}

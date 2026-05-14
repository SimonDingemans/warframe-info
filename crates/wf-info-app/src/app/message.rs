use wf_info_core::{AppSettings, ScanKind, ScanOutput};

use crate::hotkeys::HotkeyEvent;

#[derive(Debug, Clone)]
pub(super) enum Message {
    RewardScanChanged(String),
    InventoryScanChanged(String),
    Save,
    ResetDefaults,
    ConfigureHotkeysRequested,
    ConfigureHotkeysFinished(AppSettings, Result<String, String>),
    Hotkey(HotkeyEvent),
    RewardScanRequested,
    InventoryScanRequested,
    ScanFinished(ScanKind, Result<ScanOutput, String>),
}

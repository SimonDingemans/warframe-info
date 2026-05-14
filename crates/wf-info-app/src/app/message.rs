use wf_info_core::{AppSettings, ScanKind};

use crate::{hotkeys::HotkeyEvent, scan::ScanReport};

#[derive(Debug, Clone)]
pub(super) enum Message {
    RewardScanChanged(String),
    InventoryScanChanged(String),
    Save,
    ResetDefaults,
    ConfigureHotkeysRequested,
    ConfigureHotkeysFinished(AppSettings, Result<String, String>),
    Hotkey(HotkeyEvent),
    ScreenCapturePermissionFinished(Result<(), String>),
    ResetScreenCaptureTokenRequested,
    TestOverlayRequested,
    RewardScanRequested,
    InventoryScanRequested,
    ScanFinished(ScanKind, Result<ScanReport, String>),
}

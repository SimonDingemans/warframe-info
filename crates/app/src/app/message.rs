use hotkeys::HotkeyEvent;
use info_core::{AppSettings, ScanKind};

use crate::scan::ScanReport;

use super::state::AppTab;

#[derive(Debug, Clone)]
pub(super) enum Message {
    TabSelected(AppTab),
    RewardScanChanged(String),
    InventoryScanChanged(String),
    ShowRewardOverlayChanged(bool),
    Save,
    ResetDefaults,
    ConfigureHotkeysRequested,
    ConfigureHotkeysFinished(AppSettings, Result<String, String>),
    Hotkey(HotkeyEvent),
    ScreenCapturePermissionFinished(Result<(), String>),
    ResetScreenCaptureTokenRequested,
    InvalidateMarketCacheRequested,
    TestOverlayRequested,
    RewardScanRequested,
    InventoryScanRequested,
    ScanFinished(ScanKind, Result<ScanReport, String>),
}

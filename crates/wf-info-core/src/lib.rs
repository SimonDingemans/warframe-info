pub mod platform;
pub mod scan;
pub mod settings;

pub use scan::{scan_image, ScanError, ScanKind, ScanOutput, ScanResult};
pub use settings::{AppSettings, HotkeySettings, SettingsError, SettingsPaths, SettingsResult};

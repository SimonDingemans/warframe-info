use crate::scan::ScanKind;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Triggered(ScanKind),
    Status(String),
}

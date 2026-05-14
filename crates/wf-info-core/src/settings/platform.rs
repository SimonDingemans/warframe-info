#[cfg(target_os = "linux")]
#[path = "platform/linux.rs"]
mod imp;
#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod imp;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[path = "platform/unsupported.rs"]
mod imp;

use std::path::PathBuf;

pub(super) fn config_dir() -> PathBuf {
    imp::config_dir()
}

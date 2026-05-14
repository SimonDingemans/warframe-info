#[cfg(target_os = "linux")]
#[path = "platform/linux.rs"]
mod imp;
#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod imp;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[path = "platform/unsupported.rs"]
mod imp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalShortcutBackend {
    Native,
    SystemConfigured,
}

pub trait Platform: Sync {
    fn global_shortcut_backend(&self) -> GlobalShortcutBackend;
}

pub fn current() -> &'static dyn Platform {
    &imp::PLATFORM
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod unsupported;

#[cfg(target_os = "linux")]
pub(super) use linux::*;

#[cfg(target_os = "windows")]
pub(super) use windows::*;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub(super) use unsupported::*;

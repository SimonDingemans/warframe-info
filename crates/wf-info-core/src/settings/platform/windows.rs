use std::{env, path::PathBuf};

pub(super) fn config_dir() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("AppData/Roaming"))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

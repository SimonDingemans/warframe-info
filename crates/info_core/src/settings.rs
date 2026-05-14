use std::{
    fs, io,
    path::{Path, PathBuf},
};

mod platform;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type SettingsResult<T> = Result<T, SettingsError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppSettings {
    pub hotkeys: HotkeySettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkeys: HotkeySettings::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HotkeySettings {
    pub reward_scan: String,
    pub inventory_scan: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            reward_scan: "Ctrl+Shift+R".to_owned(),
            inventory_scan: "Ctrl+Shift+I".to_owned(),
        }
    }
}

impl AppSettings {
    pub fn load_or_create(path: impl AsRef<Path>) -> SettingsResult<Self> {
        let path = path.as_ref();

        if path.exists() {
            return Self::load(path);
        }

        let settings = Self::default();
        settings.save(path)?;
        Ok(settings)
    }

    pub fn load(path: impl AsRef<Path>) -> SettingsResult<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| SettingsError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&contents).map_err(|source| SettingsError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn save(&self, path: impl AsRef<Path>) -> SettingsResult<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| SettingsError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let contents = toml::to_string_pretty(self).map_err(SettingsError::Serialize)?;
        fs::write(path, contents).map_err(|source| SettingsError::Write {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPaths {
    pub settings_file: PathBuf,
}

impl SettingsPaths {
    pub fn detect() -> Self {
        Self {
            settings_file: default_settings_path(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("failed to read settings from {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse settings from {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to serialize settings")]
    Serialize(#[source] toml::ser::Error),

    #[error("failed to create settings directory {}", path.display())]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write settings to {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

fn default_settings_path() -> PathBuf {
    platform::config_dir()
        .join("warframe-info")
        .join("settings.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_round_trip_as_toml() {
        let settings = AppSettings::default();
        let serialized = toml::to_string_pretty(&settings).unwrap();

        assert!(serialized.contains("[hotkeys]"));
        assert!(serialized.contains("reward_scan = \"Ctrl+Shift+R\""));
        assert!(serialized.contains("inventory_scan = \"Ctrl+Shift+I\""));

        let parsed: AppSettings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed, settings);
    }

    #[test]
    fn load_or_create_writes_missing_settings_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("warframe-info/settings.toml");

        let settings = AppSettings::load_or_create(&path).unwrap();

        assert_eq!(settings, AppSettings::default());
        assert!(path.exists());
        assert_eq!(AppSettings::load(&path).unwrap(), settings);
    }
}

use std::path::PathBuf;

use hotkeys::HotkeyBindings;
use iced::Subscription;
use info_core::AppSettings;

use super::message::Message;
use crate::system_hotkeys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AppTab {
    Settings,
    Scan,
}

pub(super) struct SettingsApp {
    pub(super) settings_path: PathBuf,
    pub(super) active_settings: AppSettings,
    pub(super) active_tab: AppTab,
    pub(super) reward_scan: String,
    pub(super) inventory_scan: String,
    pub(super) is_dirty: bool,
    pub(super) is_scanning: bool,
    pub(super) last_scan: Option<info_core::ScanOutput>,
    pub(super) hotkeys: HotkeyBindings,
    pub(super) hotkey_status: String,
    pub(super) status: String,
}

impl SettingsApp {
    pub(super) fn load(settings_path: PathBuf) -> Self {
        match AppSettings::load_or_create(&settings_path) {
            Ok(settings) => {
                let (hotkeys, hotkey_status) =
                    HotkeyBindings::new(&settings, system_hotkeys::hotkey_backend());

                Self {
                    settings_path,
                    active_settings: settings.clone(),
                    active_tab: AppTab::Scan,
                    reward_scan: settings.hotkeys.reward_scan,
                    inventory_scan: settings.hotkeys.inventory_scan,
                    is_dirty: false,
                    is_scanning: false,
                    last_scan: None,
                    hotkeys,
                    hotkey_status,
                    status: "Settings loaded".to_owned(),
                }
            }
            Err(error) => {
                let settings = AppSettings::default();
                let (hotkeys, hotkey_status) =
                    HotkeyBindings::new(&settings, system_hotkeys::hotkey_backend());

                Self {
                    settings_path,
                    active_settings: settings.clone(),
                    active_tab: AppTab::Scan,
                    reward_scan: settings.hotkeys.reward_scan,
                    inventory_scan: settings.hotkeys.inventory_scan,
                    is_dirty: true,
                    is_scanning: false,
                    last_scan: None,
                    hotkeys,
                    hotkey_status,
                    status: format!("Could not load settings: {error}"),
                }
            }
        }
    }

    pub(super) fn settings(&self) -> AppSettings {
        AppSettings {
            hotkeys: info_core::HotkeySettings {
                reward_scan: self.reward_scan.trim().to_owned(),
                inventory_scan: self.inventory_scan.trim().to_owned(),
            },
        }
    }

    pub(super) fn subscription(&self) -> Subscription<Message> {
        self.hotkeys
            .subscription(&self.active_settings)
            .map(Message::Hotkey)
    }
}

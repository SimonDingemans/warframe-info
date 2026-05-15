use std::path::PathBuf;

use hotkeys::HotkeyBindings;
use iced::Subscription;
use info_core::AppSettings;
use ui_core::RewardCardAssets;

use super::message::Message;
use crate::system_hotkeys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AppTab {
    Settings,
    Scan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultSort {
    None,
    Platinum,
    Ducats,
    DucatsPerPlatinum,
}

impl ResultSort {
    pub(super) const ALL: [Self; 4] = [
        Self::None,
        Self::Platinum,
        Self::Ducats,
        Self::DucatsPerPlatinum,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Platinum => "Plat value",
            Self::Ducats => "Ducat value",
            Self::DucatsPerPlatinum => "Ducat / plat",
        }
    }
}

pub(super) struct SettingsApp {
    pub(super) settings_path: PathBuf,
    pub(super) active_settings: AppSettings,
    pub(super) active_tab: AppTab,
    pub(super) reward_scan: String,
    pub(super) inventory_scan: String,
    pub(super) show_reward_overlay: bool,
    pub(super) is_dirty: bool,
    pub(super) is_scanning: bool,
    pub(super) last_scan: Option<info_core::ScanOutput>,
    pub(super) result_sort: ResultSort,
    pub(super) hotkeys: HotkeyBindings,
    pub(super) hotkey_status: String,
    pub(super) status: String,
    pub(super) reward_card_assets: RewardCardAssets,
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
                    show_reward_overlay: settings.overlay.show_reward_overlay,
                    is_dirty: false,
                    is_scanning: false,
                    last_scan: None,
                    result_sort: ResultSort::None,
                    hotkeys,
                    hotkey_status,
                    status: "Settings loaded".to_owned(),
                    reward_card_assets: RewardCardAssets::load(),
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
                    show_reward_overlay: settings.overlay.show_reward_overlay,
                    is_dirty: true,
                    is_scanning: false,
                    last_scan: None,
                    result_sort: ResultSort::None,
                    hotkeys,
                    hotkey_status,
                    status: format!("Could not load settings: {error}"),
                    reward_card_assets: RewardCardAssets::load(),
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
            overlay: info_core::OverlaySettings {
                show_reward_overlay: self.show_reward_overlay,
            },
        }
    }

    pub(super) fn subscription(&self) -> Subscription<Message> {
        self.hotkeys
            .subscription(&self.active_settings)
            .map(Message::Hotkey)
    }
}

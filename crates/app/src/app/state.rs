use std::path::PathBuf;

use hotkeys::HotkeyBindings;
use iced::Subscription;
use info_core::AppSettings;
use ui_core::RewardCardAssets;

use super::message::Message;
use crate::{
    market::orders::{
        DraftMode, MarketOrder, OrderDraft, OrderItemOption, OrderSession, PendingOrderAction,
    },
    system_hotkeys,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AppTab {
    Settings,
    Scan,
    Orders,
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
    pub(super) order_session_path: PathBuf,
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
    pub(super) order_email: String,
    pub(super) order_password: String,
    pub(super) order_session: Option<OrderSession>,
    pub(super) is_order_authenticating: bool,
    pub(super) is_loading_order_items: bool,
    pub(super) is_loading_orders: bool,
    pub(super) is_mutating_order: bool,
    pub(super) orders: Vec<MarketOrder>,
    pub(super) order_items: Vec<OrderItemOption>,
    pub(super) order_search: String,
    pub(super) order_draft: OrderDraft,
    pub(super) pending_order_action: Option<PendingOrderAction>,
}

impl SettingsApp {
    pub(super) fn load(settings_path: PathBuf) -> Self {
        let order_session_path = crate::market::orders::session_path_for_settings(&settings_path);

        match AppSettings::load_or_create(&settings_path) {
            Ok(settings) => {
                let (hotkeys, hotkey_status) =
                    HotkeyBindings::new(&settings, system_hotkeys::hotkey_backend());

                Self {
                    settings_path,
                    order_session_path,
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
                    order_email: String::new(),
                    order_password: String::new(),
                    order_session: None,
                    is_order_authenticating: false,
                    is_loading_order_items: false,
                    is_loading_orders: false,
                    is_mutating_order: false,
                    orders: Vec::new(),
                    order_items: Vec::new(),
                    order_search: String::new(),
                    order_draft: OrderDraft::empty(),
                    pending_order_action: None,
                }
            }
            Err(error) => {
                let settings = AppSettings::default();
                let (hotkeys, hotkey_status) =
                    HotkeyBindings::new(&settings, system_hotkeys::hotkey_backend());

                Self {
                    settings_path,
                    order_session_path,
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
                    order_email: String::new(),
                    order_password: String::new(),
                    order_session: None,
                    is_order_authenticating: false,
                    is_loading_order_items: false,
                    is_loading_orders: false,
                    is_mutating_order: false,
                    orders: Vec::new(),
                    order_items: Vec::new(),
                    order_search: String::new(),
                    order_draft: OrderDraft::empty(),
                    pending_order_action: None,
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

    pub(super) fn draft_is_editing(&self) -> bool {
        matches!(self.order_draft.mode, DraftMode::Edit(_))
    }
}

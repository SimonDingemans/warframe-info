use iced::Task;
use info_core::{AppSettings, ScanKind};

use hotkeys::HotkeyEvent;

use crate::{
    market::orders::{self, OrderItemOption, PendingOrderAction},
    overlay::{spawn_reward_overlay, spawn_test_reward_overlay},
    scan::run_scan,
    system_hotkeys,
};

use super::{message::Message, state::SettingsApp};

impl SettingsApp {
    pub(super) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                if tab == super::state::AppTab::Orders {
                    return self.start_orders_tab();
                }
            }
            Message::ResultSortSelected(sort) => {
                self.result_sort = sort;
            }
            Message::RewardScanChanged(value) => {
                self.reward_scan = value;
                self.is_dirty = true;
                self.status = "Unsaved changes".to_owned();
            }
            Message::InventoryScanChanged(value) => {
                self.inventory_scan = value;
                self.is_dirty = true;
                self.status = "Unsaved changes".to_owned();
            }
            Message::ShowRewardOverlayChanged(value) => {
                self.show_reward_overlay = value;
                self.is_dirty = true;
                self.status = "Unsaved changes".to_owned();
            }
            Message::Save => match self.settings().save(&self.settings_path) {
                Ok(()) => {
                    let settings = self.settings();
                    self.hotkey_status = self.hotkeys.configure(&settings);
                    self.active_settings = settings;
                    self.is_dirty = false;
                    self.status = format!("Saved {}", self.settings_path.display());
                }
                Err(error) => {
                    self.status = format!("Could not save settings: {error}");
                }
            },
            Message::ResetDefaults => {
                let settings = AppSettings::default();
                self.reward_scan = settings.hotkeys.reward_scan;
                self.inventory_scan = settings.hotkeys.inventory_scan;
                self.show_reward_overlay = settings.overlay.show_reward_overlay;
                self.is_dirty = true;
                self.status = "Defaults restored".to_owned();
            }
            Message::ConfigureHotkeysRequested => {
                let settings = self.settings();

                match settings.save(&self.settings_path) {
                    Ok(()) => {
                        self.is_dirty = false;
                        self.status = format!("Saved {}", self.settings_path.display());
                    }
                    Err(error) => {
                        self.status = format!("Could not save settings: {error}");
                        return Task::none();
                    }
                }

                self.hotkey_status = "Opening system shortcut configuration".to_owned();

                let configured_settings = settings.clone();

                return Task::perform(
                    system_hotkeys::configure_system_shortcuts(settings),
                    move |result| {
                        Message::ConfigureHotkeysFinished(
                            configured_settings.clone(),
                            result.map_err(|error| error.to_string()),
                        )
                    },
                );
            }
            Message::ConfigureHotkeysFinished(settings, result) => match result {
                Ok(status) => {
                    self.hotkeys.configure(&settings);
                    self.active_settings = settings;
                    self.hotkey_status = status;
                }
                Err(error) => {
                    self.hotkey_status = format!("Could not configure desktop shortcuts: {error}");
                }
            },
            Message::Hotkey(event) => match event {
                HotkeyEvent::Triggered(kind) => return self.start_scan(kind),
                HotkeyEvent::Status(status) => {
                    self.hotkey_status = status;
                }
            },
            Message::ScreenCapturePermissionFinished(result) => match result {
                Ok(()) => {
                    self.status = "Screen capture permission ready".to_owned();
                }
                Err(error) => {
                    self.status = format!("Screen capture permission failed: {error}");
                }
            },
            Message::ResetScreenCaptureTokenRequested => {
                match crate::scan::reset_screen_capture_restore_token() {
                    Ok(()) => {
                        self.status = "Screen capture token reset".to_owned();
                    }
                    Err(error) => {
                        self.status = format!("Could not reset screen capture token: {error}");
                    }
                }
            }
            Message::InvalidateMarketCacheRequested => match crate::market::invalidate_caches() {
                Ok(()) => {
                    self.status = "Warframe Market cache cleared".to_owned();
                }
                Err(error) => {
                    self.status = format!("Could not clear Warframe Market cache: {error}");
                }
            },
            Message::TestOverlayRequested => match spawn_test_reward_overlay() {
                Ok(()) => {
                    self.status = "Test overlay spawned".to_owned();
                }
                Err(error) => {
                    self.status = format!("Test overlay failed: {error}");
                }
            },
            Message::RewardScanRequested => {
                return self.start_scan(ScanKind::Reward);
            }
            Message::InventoryScanRequested => {
                return self.start_scan(ScanKind::Inventory);
            }
            Message::ScanFinished(kind, result) => {
                self.is_scanning = false;

                match result {
                    Ok(report) => {
                        let output = report.output;
                        let item_count = output.items.len();
                        let overlay_status = if self.show_reward_overlay {
                            spawn_reward_overlay(&output, report.overlay_output_size)
                                .err()
                                .map(|error| format!("; overlay failed: {error}"))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        self.status = format!(
                            "{} scan found {item_count} item{} from {} text block{}",
                            output.kind.label(),
                            plural_suffix(item_count),
                            output.text_block_count,
                            plural_suffix(output.text_block_count),
                        );
                        self.status.push_str(&overlay_status);
                        self.last_scan = Some(output);
                    }
                    Err(error) => {
                        self.status = format!("{} scan failed: {error}", kind.label());
                    }
                }
            }
            Message::RestoreOrderSessionFinished(result) => {
                self.is_order_authenticating = false;

                match result {
                    Ok(Some(session)) => {
                        self.order_email = session.email.clone();
                        self.order_session = Some(session);
                        self.status = "Warframe Market session restored".to_owned();
                        return self.refresh_orders();
                    }
                    Ok(None) => {
                        self.status = "Log in to manage Warframe Market orders".to_owned();
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
            Message::OrderEmailChanged(value) => {
                self.order_email = value;
            }
            Message::OrderPasswordChanged(value) => {
                self.order_password = value;
            }
            Message::OrderLoginRequested => {
                if self.is_order_authenticating {
                    return Task::none();
                }

                self.is_order_authenticating = true;
                self.status = "Logging in to Warframe Market".to_owned();

                let session_path = self.order_session_path.clone();
                let email = self.order_email.clone();
                let password = self.order_password.clone();

                return Task::perform(orders::login(session_path, email, password), |result| {
                    Message::OrderLoginFinished(result)
                });
            }
            Message::OrderLoginFinished(result) => {
                self.is_order_authenticating = false;
                self.order_password.clear();

                match result {
                    Ok(session) => {
                        self.order_email = session.email.clone();
                        self.order_session = Some(session);
                        self.status = "Logged in to Warframe Market".to_owned();
                        return self.refresh_orders();
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
            Message::OrderLogoutRequested => match orders::logout(&self.order_session_path) {
                Ok(()) => {
                    self.order_session = None;
                    self.orders.clear();
                    self.order_password.clear();
                    self.pending_order_action = None;
                    self.status = "Logged out of Warframe Market".to_owned();
                }
                Err(error) => {
                    self.status = error;
                }
            },
            Message::OrderItemsLoaded(result) => {
                self.is_loading_order_items = false;

                match result {
                    Ok(items) => {
                        let count = items.len();
                        self.order_items = items;
                        self.status = format!("Loaded {count} Warframe Market items");
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
            Message::OrdersRefreshRequested => {
                return self.refresh_orders();
            }
            Message::OrdersLoaded(result) => {
                self.is_loading_orders = false;

                match result {
                    Ok(orders) => {
                        let count = orders.len();
                        self.orders = orders;
                        self.status = format!(
                            "Loaded {count} Warframe Market order{}",
                            plural_suffix(count)
                        );
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
            Message::OrderSearchChanged(value) => {
                self.order_search = value;
            }
            Message::ManualOrderDraftRequested(item, side) => {
                self.status = format!("Preparing {} order for {}", side.label(), item.name);
                return Task::perform(
                    orders::create_draft_with_price(item, side, None),
                    Message::OrderDraftLoaded,
                );
            }
            Message::ScanItemOrderDraftRequested(index, side) => {
                let Some(output) = self.last_scan.as_ref() else {
                    self.status = "No scan result item to order".to_owned();
                    return Task::none();
                };

                let Some(item) = output.items.get(index).cloned() else {
                    self.status = "Scan result item is no longer available".to_owned();
                    return Task::none();
                };

                let Some(option) = OrderItemOption::from_scan_item(&item) else {
                    self.status = format!("{} is not linked to Warframe Market", item.name);
                    return Task::none();
                };
                let option = self
                    .order_items
                    .iter()
                    .find(|candidate| candidate.slug == option.slug)
                    .cloned()
                    .unwrap_or(option);

                let fallback_price =
                    (item.platinum_rounded() > 0).then_some(item.platinum_rounded());
                self.active_tab = super::state::AppTab::Orders;
                self.status = format!("Preparing {} order for {}", side.label(), option.name);

                let draft_task = Task::perform(
                    orders::create_draft_with_price(option, side, fallback_price),
                    Message::OrderDraftLoaded,
                );
                let tab_task = self.start_orders_tab();

                return Task::batch([draft_task, tab_task]);
            }
            Message::OrderDraftLoaded(draft) => {
                self.order_search = draft.item_name.clone();
                self.order_draft = draft;
                self.pending_order_action = None;
                self.status = "Order draft ready".to_owned();
            }
            Message::OrderDraftSideChanged(side) => {
                self.order_draft.side = side;
                if matches!(self.order_draft.mode, orders::DraftMode::Create)
                    && !self.order_draft.item_slug.is_empty()
                {
                    let draft = self.order_draft.clone();
                    return Task::perform(
                        orders::refresh_draft_price(draft, None),
                        Message::OrderDraftLoaded,
                    );
                }
            }
            Message::OrderDraftPriceChanged(value) => {
                self.order_draft.platinum = value;
            }
            Message::OrderDraftQuantityChanged(value) => {
                self.order_draft.quantity = value;
            }
            Message::OrderDraftVisibleChanged(value) => {
                self.order_draft.visible = value;
            }
            Message::OrderDraftRankChanged(value) => {
                self.order_draft.rank = value;
                if matches!(self.order_draft.mode, orders::DraftMode::Create)
                    && !self.order_draft.item_slug.is_empty()
                {
                    let draft = self.order_draft.clone();
                    return Task::perform(
                        orders::refresh_draft_price(draft, None),
                        Message::OrderDraftLoaded,
                    );
                }
            }
            Message::OrderDraftChargesChanged(value) => {
                self.order_draft.charges = value;
            }
            Message::OrderDraftAmberStarsChanged(value) => {
                self.order_draft.amber_stars = value;
            }
            Message::OrderDraftCyanStarsChanged(value) => {
                self.order_draft.cyan_stars = value;
            }
            Message::OrderDraftSubtypeChanged(value) => {
                self.order_draft.subtype = value;
            }
            Message::OrderEditRequested(order_id) => {
                if let Some(order) = self.orders.iter().find(|order| order.id == order_id) {
                    self.order_search = order.item_name.clone();
                    self.order_draft = orders::OrderDraft::edit(order);
                    self.pending_order_action = None;
                    self.status = "Order loaded into editor".to_owned();
                }
            }
            Message::OrderDeleteRequested(order_id) => {
                self.pending_order_action = Some(PendingOrderAction::Delete { order_id });
            }
            Message::OrderCloseRequested(order_id) => {
                if let Some(order) = self.orders.iter().find(|order| order.id == order_id) {
                    self.pending_order_action = Some(PendingOrderAction::Close {
                        order_id,
                        quantity: order.quantity,
                    });
                }
            }
            Message::OrderSubmitRequested => {
                if self.order_session.is_none() {
                    self.status = "Log in before submitting Warframe Market orders".to_owned();
                    return Task::none();
                }

                match orders::pending_action_from_draft(&self.order_draft) {
                    Ok(action) => {
                        self.pending_order_action = Some(action);
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
            Message::OrderMutationConfirmed => {
                if self.is_mutating_order {
                    return Task::none();
                }

                let Some(session) = self.order_session.as_ref() else {
                    self.status = "Log in before changing Warframe Market orders".to_owned();
                    return Task::none();
                };

                let Some(action) = self.pending_order_action.clone() else {
                    return Task::none();
                };

                self.is_mutating_order = true;
                self.status = "Updating Warframe Market order".to_owned();

                return Task::perform(
                    orders::commit_action(session.client.clone(), action),
                    Message::OrderMutationFinished,
                );
            }
            Message::OrderMutationCanceled => {
                self.pending_order_action = None;
            }
            Message::OrderMutationFinished(result) => {
                self.is_mutating_order = false;
                self.pending_order_action = None;

                match result {
                    Ok(orders) => {
                        self.orders = orders;
                        self.order_draft = orders::OrderDraft::empty();
                        self.status = "Warframe Market order updated".to_owned();
                    }
                    Err(error) => {
                        self.status = error;
                    }
                }
            }
        }

        Task::none()
    }

    fn start_orders_tab(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();

        if self.order_items.is_empty() && !self.is_loading_order_items {
            self.is_loading_order_items = true;
            tasks.push(Task::perform(
                orders::load_item_options(),
                Message::OrderItemsLoaded,
            ));
        }

        if self.order_session.is_none() && !self.is_order_authenticating {
            self.is_order_authenticating = true;
            let session_path = self.order_session_path.clone();
            tasks.push(Task::perform(
                orders::restore_session(session_path),
                Message::RestoreOrderSessionFinished,
            ));
        } else if self.order_session.is_some() && self.orders.is_empty() && !self.is_loading_orders
        {
            tasks.push(self.refresh_orders());
        }

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    fn refresh_orders(&mut self) -> Task<Message> {
        if self.is_loading_orders {
            return Task::none();
        }

        let Some(session) = self.order_session.as_ref() else {
            self.status = "Log in to refresh Warframe Market orders".to_owned();
            return Task::none();
        };

        self.is_loading_orders = true;
        self.status = "Loading Warframe Market orders".to_owned();

        Task::perform(
            orders::load_orders(session.client.clone()),
            Message::OrdersLoaded,
        )
    }

    fn start_scan(&mut self, kind: ScanKind) -> Task<Message> {
        if self.is_scanning {
            return Task::none();
        }

        self.is_scanning = true;
        self.last_scan = None;
        self.status = match kind {
            ScanKind::Reward => "Scanning reward screen".to_owned(),
            ScanKind::Inventory => "Scanning inventory".to_owned(),
        };

        Task::perform(run_scan(kind), move |result| {
            Message::ScanFinished(kind, result)
        })
    }
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

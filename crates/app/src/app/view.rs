use iced::{
    alignment,
    widget::{
        button, column, container, row, rule, scrollable,
        scrollable::{Direction, Scrollbar},
        text, text_input, toggler,
    },
    Element, Length,
};
use ui_core::reward_cards_from_scan_output;

use crate::{
    market::orders::{self, DraftMode, OrderSide},
    scan, system_hotkeys,
};

use super::{
    message::Message,
    state::{AppTab, ResultSort, SettingsApp},
};

impl SettingsApp {
    pub(super) fn view(&self) -> Element<'_, Message> {
        let title = column![
            text("Warframe Info").size(30),
            text(status_text(self)).size(14),
        ]
        .spacing(4);

        let tabs = row![
            tab_button("Scan", AppTab::Scan, self.active_tab),
            tab_button("Orders", AppTab::Orders, self.active_tab),
            tab_button("Settings", AppTab::Settings, self.active_tab),
        ]
        .spacing(10);

        let content = match self.active_tab {
            AppTab::Settings => self.settings_tab(),
            AppTab::Scan => self.scan_tab(),
            AppTab::Orders => self.orders_tab(),
        };

        container(column![title, tabs, rule::horizontal(1), content].spacing(18))
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings_tab(&self) -> Element<'_, Message> {
        let hotkeys = column![
            text("Hotkeys").size(22),
            text(self.settings_path.display().to_string()).size(14),
            labeled_input(
                "Reward scan",
                "Ctrl+Shift+R",
                &self.reward_scan,
                Message::RewardScanChanged,
            ),
            labeled_input(
                "Inventory scan",
                "Ctrl+Shift+I",
                &self.inventory_scan,
                Message::InventoryScanChanged,
            ),
        ]
        .spacing(12);

        let overlay = column![
            text("Overlay").size(22),
            toggler(self.show_reward_overlay)
                .label("Show reward overlay after reward scans")
                .on_toggle(Message::ShowRewardOverlayChanged)
                .size(18),
        ]
        .spacing(12);

        let mut actions = row![
            button(icon_label("✓", "Save"))
                .on_press(Message::Save)
                .padding([8, 14]),
            button(icon_label("↺", "Reset"))
                .on_press(Message::ResetDefaults)
                .padding([8, 14]),
        ]
        .spacing(10);

        if system_hotkeys::has_system_shortcut_configuration() {
            actions = actions.push(
                button(icon_label("⌘", "Configure Hotkeys"))
                    .on_press(Message::ConfigureHotkeysRequested)
                    .padding([8, 14]),
            );
        }

        if scan::should_request_screen_capture_permission() {
            actions = actions.push(
                button(icon_label("↺", "Reset Screen Token"))
                    .on_press(Message::ResetScreenCaptureTokenRequested)
                    .padding([8, 14]),
            );
        }

        column![
            hotkeys,
            overlay,
            actions,
            text(&self.hotkey_status).size(14)
        ]
        .spacing(18)
        .into()
    }

    fn scan_tab(&self) -> Element<'_, Message> {
        let reward_scan_button = if self.is_scanning {
            button(icon_label("◎", "Reward Scan")).padding([8, 14])
        } else {
            button(icon_label("◎", "Reward Scan"))
                .on_press(Message::RewardScanRequested)
                .padding([8, 14])
        };
        let inventory_scan_button = if self.is_scanning {
            button(icon_label("▦", "Inventory Scan")).padding([8, 14])
        } else {
            button(icon_label("▦", "Inventory Scan"))
                .on_press(Message::InventoryScanRequested)
                .padding([8, 14])
        };

        let pipeline_actions = row![
            reward_scan_button,
            inventory_scan_button,
            button(icon_label("⌫", "Clear Market Cache"))
                .on_press(Message::InvalidateMarketCacheRequested)
                .padding([8, 14]),
            button(icon_label("□", "Test Overlay"))
                .on_press(Message::TestOverlayRequested)
                .padding([8, 14]),
        ]
        .spacing(10);

        let results = scan_results(self);

        column![pipeline_actions, results]
            .spacing(18)
            .height(Length::Fill)
            .into()
    }

    fn orders_tab(&self) -> Element<'_, Message> {
        let auth = order_auth_panel(self);
        let search = order_search_panel(self);
        let draft = order_draft_panel(self);
        let orders = my_orders_panel(self);

        let mut content = column![auth, rule::horizontal(1), search, draft]
            .spacing(16)
            .width(Length::Fill);

        if let Some(action) = &self.pending_order_action {
            content = content.push(order_confirmation_panel(action.description()));
        }

        content = content.push(orders);

        vertical_scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn tab_button(label: &str, tab: AppTab, active_tab: AppTab) -> Element<'_, Message> {
    let style = if tab == active_tab {
        iced::widget::button::primary
    } else {
        iced::widget::button::secondary
    };

    button(label)
        .on_press(Message::TabSelected(tab))
        .padding([8, 14])
        .style(style)
        .into()
}

fn status_text(app: &SettingsApp) -> String {
    if app.is_dirty {
        format!("{} - not saved", app.status)
    } else {
        app.status.clone()
    }
}

fn scan_results(app: &SettingsApp) -> Element<'_, Message> {
    let body = scan_results_body(app);
    let header = row![
        text("Results").size(22).width(Length::Fill),
        result_sort_controls(app.result_sort),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    column![header, body].spacing(8).height(Length::Fill).into()
}

fn scan_results_body(app: &SettingsApp) -> Element<'_, Message> {
    let Some(output) = app.last_scan.as_ref() else {
        return centered_results_text("No scan results yet");
    };

    let mut items = reward_cards_from_scan_output(output);

    if items.is_empty() {
        return centered_results_text("No items found");
    }

    sort_reward_cards(&mut items, app.result_sort);

    let cards = ui_core::reward_cards_row(items, &app.reward_card_assets)
        .wrap()
        .vertical_spacing(ui_core::REWARD_CARD_SPACING)
        .align_x(alignment::Horizontal::Center);

    let centered_cards = container(cards).center(Length::Fill);
    let actions = scan_order_actions(app);

    vertical_scrollable(column![centered_cards, actions].spacing(16))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn scan_order_actions(app: &SettingsApp) -> Element<'_, Message> {
    let Some(output) = app.last_scan.as_ref() else {
        return column![].into();
    };

    let mut list = column![text("Create Warframe Market orders").size(18)].spacing(8);

    for (index, item) in output.items.iter().enumerate() {
        if item.market_slug.is_none() {
            continue;
        }

        list = list.push(
            row![
                text(&item.name).width(Length::Fill),
                button(icon_label("↑", "Sell"))
                    .on_press(Message::ScanItemOrderDraftRequested(index, OrderSide::Sell))
                    .padding([6, 10]),
                button(icon_label("↓", "Buy"))
                    .on_press(Message::ScanItemOrderDraftRequested(index, OrderSide::Buy))
                    .padding([6, 10]),
            ]
            .spacing(8)
            .align_y(alignment::Vertical::Center),
        );
    }

    list.into()
}

fn order_auth_panel(app: &SettingsApp) -> Element<'_, Message> {
    if let Some(session) = &app.order_session {
        let refresh = if app.is_loading_orders {
            button(icon_label("↻", "Refresh")).padding([8, 14])
        } else {
            button(icon_label("↻", "Refresh"))
                .on_press(Message::OrdersRefreshRequested)
                .padding([8, 14])
        };

        return column![
            text("Warframe Market Account").size(22),
            row![
                text(format!("Logged in as {}", session.email)).width(Length::Fill),
                refresh,
                button(icon_label("⇤", "Logout"))
                    .on_press(Message::OrderLogoutRequested)
                    .padding([8, 14]),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
        ]
        .spacing(10)
        .into();
    }

    let login_button = if app.is_order_authenticating {
        button(icon_label("⇥", "Login")).padding([8, 14])
    } else {
        button(icon_label("⇥", "Login"))
            .on_press(Message::OrderLoginRequested)
            .padding([8, 14])
    };

    column![
        text("Warframe Market Account").size(22),
        row![
            text_input("Email or username", &app.order_email)
                .on_input(Message::OrderEmailChanged)
                .padding(8)
                .width(Length::Fill),
            text_input("Password", &app.order_password)
                .on_input(Message::OrderPasswordChanged)
                .secure(true)
                .padding(8)
                .width(Length::Fill),
            login_button,
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
        text(app.order_session_path.display().to_string()).size(12),
    ]
    .spacing(10)
    .into()
}

fn order_search_panel(app: &SettingsApp) -> Element<'_, Message> {
    let mut content = column![
        text("Manual Order").size(22),
        text_input("Search tradable item", &app.order_search)
            .on_input(Message::OrderSearchChanged)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(10);

    if app.is_loading_order_items {
        content = content.push(text("Loading tradable items").size(14));
    } else {
        let matches = orders::search_item_options(&app.order_items, &app.order_search, 8);

        for item in matches {
            content = content.push(
                row![
                    text(item.name.clone()).width(Length::Fill),
                    button(icon_label("↑", "Sell"))
                        .on_press(Message::ManualOrderDraftRequested(
                            item.clone(),
                            OrderSide::Sell
                        ))
                        .padding([6, 10]),
                    button(icon_label("↓", "Buy"))
                        .on_press(Message::ManualOrderDraftRequested(item, OrderSide::Buy))
                        .padding([6, 10]),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center),
            );
        }
    }

    content.into()
}

fn order_draft_panel(app: &SettingsApp) -> Element<'_, Message> {
    let draft = &app.order_draft;
    let title = if app.draft_is_editing() {
        "Edit Order"
    } else {
        "Order Draft"
    };

    let mut fields = column![row![
        text(title).size(22).width(Length::Fill),
        text(if draft.item_name.is_empty() {
            "No item selected".to_owned()
        } else {
            format!("{} {}", draft.side.label(), draft.item_name)
        })
        .size(14),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center),]
    .spacing(10);

    if matches!(draft.mode, DraftMode::Create) && !draft.item_name.is_empty() {
        fields = fields.push(order_side_controls(draft.side));
    }

    let mut primary_fields = row![
        labeled_order_input(
            "Price",
            "Platinum",
            &draft.platinum,
            Message::OrderDraftPriceChanged,
        ),
        labeled_order_input(
            "Quantity",
            "1",
            &draft.quantity,
            Message::OrderDraftQuantityChanged,
        ),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    if draft.capabilities.max_rank.is_some() || !draft.rank.is_empty() {
        primary_fields = primary_fields.push(labeled_order_input(
            "Rank",
            "0",
            &draft.rank,
            Message::OrderDraftRankChanged,
        ));
    }

    primary_fields = primary_fields.push(
        toggler(draft.visible)
            .label("Visible")
            .on_toggle(Message::OrderDraftVisibleChanged)
            .size(18),
    );

    fields = fields.push(primary_fields);

    let mut optional = row![].spacing(12).align_y(alignment::Vertical::Center);
    let mut has_optional = false;

    if draft.capabilities.max_charges.is_some() || !draft.charges.is_empty() {
        optional = optional.push(labeled_order_input(
            "Charges",
            "0",
            &draft.charges,
            Message::OrderDraftChargesChanged,
        ));
        has_optional = true;
    }

    if draft.capabilities.max_amber_stars.is_some() || !draft.amber_stars.is_empty() {
        optional = optional.push(labeled_order_input(
            "Amber",
            "0",
            &draft.amber_stars,
            Message::OrderDraftAmberStarsChanged,
        ));
        has_optional = true;
    }

    if draft.capabilities.max_cyan_stars.is_some() || !draft.cyan_stars.is_empty() {
        optional = optional.push(labeled_order_input(
            "Cyan",
            "0",
            &draft.cyan_stars,
            Message::OrderDraftCyanStarsChanged,
        ));
        has_optional = true;
    }

    if !draft.capabilities.subtypes.is_empty() || !draft.subtype.is_empty() {
        optional = optional.push(labeled_order_input(
            "Subtype",
            "blueprint",
            &draft.subtype,
            Message::OrderDraftSubtypeChanged,
        ));
        has_optional = true;
    }

    if has_optional {
        fields = fields.push(optional);
    }

    let submit_label = if app.draft_is_editing() {
        "Review Update"
    } else {
        "Review Create"
    };

    let submit = if app.order_session.is_some() && !app.is_mutating_order {
        button(icon_label("✓", submit_label))
            .on_press(Message::OrderSubmitRequested)
            .padding([8, 14])
    } else {
        button(icon_label("✓", submit_label)).padding([8, 14])
    };

    fields = fields.push(row![submit].spacing(10));

    fields.into()
}

fn order_side_controls(active: OrderSide) -> Element<'static, Message> {
    OrderSide::ALL
        .into_iter()
        .fold(row![text("Side").size(14)].spacing(8), |row, side| {
            let style = if side == active {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            };

            row.push(
                button(icon_label("•", side.label()))
                    .on_press(Message::OrderDraftSideChanged(side))
                    .padding([6, 10])
                    .style(style),
            )
        })
        .into()
}

fn my_orders_panel(app: &SettingsApp) -> Element<'_, Message> {
    let mut content = column![text("My Orders").size(22)].spacing(10);

    if app.is_loading_orders {
        return content.push(text("Loading orders").size(14)).into();
    }

    if app.order_session.is_none() {
        return content
            .push(text("Log in to load your Warframe Market orders").size(14))
            .into();
    }

    if app.orders.is_empty() {
        return content.push(text("No orders loaded").size(14)).into();
    }

    for order in &app.orders {
        let visibility = if order.visible { "Visible" } else { "Hidden" };
        let detail = order_detail_text(order);

        content = content.push(
            row![
                column![
                    text(format!(
                        "{} {} - {}p x{}",
                        order.side.label(),
                        order.item_name,
                        order.platinum,
                        order.quantity
                    )),
                    text(format!("{visibility}{detail}")).size(12),
                ]
                .spacing(2)
                .width(Length::Fill),
                button(icon_label("✎", "Edit"))
                    .on_press(Message::OrderEditRequested(order.id.clone()))
                    .padding([6, 10]),
                button(icon_label("□", "Close"))
                    .on_press(Message::OrderCloseRequested(order.id.clone()))
                    .padding([6, 10]),
                button(icon_label("✕", "Delete"))
                    .on_press(Message::OrderDeleteRequested(order.id.clone()))
                    .padding([6, 10])
                    .style(iced::widget::button::danger),
            ]
            .spacing(8)
            .align_y(alignment::Vertical::Center),
        );
    }

    content.into()
}

fn order_detail_text(order: &orders::MarketOrder) -> String {
    let mut parts = Vec::new();

    if let Some(rank) = order.rank {
        parts.push(format!("rank {rank}"));
    }

    if let Some(charges) = order.charges {
        parts.push(format!("charges {charges}"));
    }

    if let Some(amber) = order.amber_stars {
        parts.push(format!("amber {amber}"));
    }

    if let Some(cyan) = order.cyan_stars {
        parts.push(format!("cyan {cyan}"));
    }

    if let Some(subtype) = &order.subtype {
        parts.push(subtype.clone());
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!(" - {}", parts.join(", "))
    }
}

fn order_confirmation_panel(description: String) -> Element<'static, Message> {
    column![
        rule::horizontal(1),
        text("Confirm Warframe Market change").size(22),
        text(description).size(14),
        row![
            button(icon_label("✓", "Confirm"))
                .on_press(Message::OrderMutationConfirmed)
                .padding([8, 14]),
            button(icon_label("✕", "Cancel"))
                .on_press(Message::OrderMutationCanceled)
                .padding([8, 14]),
        ]
        .spacing(10),
    ]
    .spacing(10)
    .into()
}

fn result_sort_controls(active_sort: ResultSort) -> Element<'static, Message> {
    ResultSort::ALL
        .into_iter()
        .fold(
            row![text("Sort").size(14)]
                .spacing(8)
                .align_y(alignment::Vertical::Center),
            |row, sort| row.push(result_sort_button(sort, active_sort)),
        )
        .into()
}

fn result_sort_button(sort: ResultSort, active_sort: ResultSort) -> Element<'static, Message> {
    let style = if sort == active_sort {
        iced::widget::button::primary
    } else {
        iced::widget::button::secondary
    };

    button(sort.label())
        .on_press(Message::ResultSortSelected(sort))
        .padding([6, 10])
        .style(style)
        .into()
}

fn sort_reward_cards(items: &mut [ui_core::RewardCardEntry], sort: ResultSort) {
    if sort == ResultSort::None {
        return;
    }

    items.sort_by(|left, right| {
        let left_value = reward_sort_value(left, sort);
        let right_value = reward_sort_value(right, sort);

        match (left_value, right_value) {
            (Some(left), Some(right)) => right.total_cmp(&left),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
}

fn reward_sort_value(reward: &ui_core::RewardCardEntry, sort: ResultSort) -> Option<f32> {
    match sort {
        ResultSort::None => None,
        ResultSort::Platinum => reward.platinum.map(|platinum| platinum as f32),
        ResultSort::Ducats => reward.ducats.map(|ducats| ducats as f32),
        ResultSort::DucatsPerPlatinum => {
            reward
                .ducats
                .zip(reward.platinum)
                .and_then(|(ducats, platinum)| {
                    (platinum > 0).then_some(ducats as f32 / platinum as f32)
                })
        }
    }
}

fn centered_results_text(label: &'static str) -> Element<'static, Message> {
    container(text(label).size(14)).center(Length::Fill).into()
}

fn vertical_scrollable<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Scrollable<'a, Message> {
    scrollable(content).direction(Direction::Vertical(scrollbar_with_gutter()))
}

fn scrollbar_with_gutter() -> Scrollbar {
    Scrollbar::new().width(8).scroller_width(8).spacing(12)
}

fn icon_label<'a>(icon: &'a str, label: &'a str) -> Element<'a, Message> {
    row![text(icon).size(14), text(label)]
        .spacing(6)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn labeled_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label)
            .width(Length::Fixed(140.0))
            .align_y(alignment::Vertical::Center),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn labeled_order_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(12),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding(8)
            .width(Length::Fixed(120.0)),
    ]
    .spacing(4)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{sort_reward_cards, ResultSort};

    #[test]
    fn result_sort_none_keeps_scan_order() {
        let mut items = vec![
            ui_core::RewardCardEntry::name_only("Forma Blueprint").with_platinum(8),
            ui_core::RewardCardEntry::name_only("Braton Prime Receiver").with_platinum(42),
            ui_core::RewardCardEntry::name_only("Paris Prime String").with_platinum(15),
        ];

        sort_reward_cards(&mut items, ResultSort::None);

        assert_eq!(items[0].name, "Forma Blueprint");
        assert_eq!(items[1].name, "Braton Prime Receiver");
        assert_eq!(items[2].name, "Paris Prime String");
    }

    #[test]
    fn result_sort_sorts_by_highest_platinum_first() {
        let mut items = vec![
            ui_core::RewardCardEntry::name_only("Forma Blueprint").with_platinum(8),
            ui_core::RewardCardEntry::name_only("Braton Prime Receiver").with_platinum(42),
            ui_core::RewardCardEntry::name_only("Paris Prime String").with_platinum(15),
        ];

        sort_reward_cards(&mut items, ResultSort::Platinum);

        assert_eq!(items[0].name, "Braton Prime Receiver");
        assert_eq!(items[1].name, "Paris Prime String");
        assert_eq!(items[2].name, "Forma Blueprint");
    }

    #[test]
    fn result_sort_sorts_by_highest_ducat_value_first() {
        let mut items = vec![
            ui_core::RewardCardEntry::name_only("Paris Prime String").with_ducats(25),
            ui_core::RewardCardEntry::name_only("Braton Prime Receiver").with_ducats(45),
            ui_core::RewardCardEntry::name_only("Forma Blueprint").with_ducats(15),
        ];

        sort_reward_cards(&mut items, ResultSort::Ducats);

        assert_eq!(items[0].name, "Braton Prime Receiver");
        assert_eq!(items[1].name, "Paris Prime String");
        assert_eq!(items[2].name, "Forma Blueprint");
    }

    #[test]
    fn result_sort_sorts_by_highest_ducat_per_platinum_first() {
        let mut items = vec![
            ui_core::RewardCardEntry::name_only("Cheap Ducats")
                .with_platinum(5)
                .with_ducats(45),
            ui_core::RewardCardEntry::name_only("Expensive Ducats")
                .with_platinum(20)
                .with_ducats(100),
            ui_core::RewardCardEntry::name_only("No Price").with_ducats(100),
            ui_core::RewardCardEntry::name_only("Zero Price")
                .with_platinum(0)
                .with_ducats(100),
        ];

        sort_reward_cards(&mut items, ResultSort::DucatsPerPlatinum);

        assert_eq!(items[0].name, "Cheap Ducats");
        assert_eq!(items[1].name, "Expensive Ducats");
        assert_eq!(items[2].name, "No Price");
        assert_eq!(items[3].name, "Zero Price");
    }
}

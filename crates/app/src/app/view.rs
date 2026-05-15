use iced::{
    alignment,
    widget::{button, column, container, row, rule, scrollable, text, text_input, toggler},
    Element, Length,
};
use ui_core::reward_cards_from_scan_output;

use crate::{scan, system_hotkeys};

use super::{
    message::Message,
    state::{AppTab, SettingsApp},
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
            tab_button("Settings", AppTab::Settings, self.active_tab),
        ]
        .spacing(10);

        let content = match self.active_tab {
            AppTab::Settings => self.settings_tab(),
            AppTab::Scan => self.scan_tab(),
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
            button("Save").on_press(Message::Save).padding([8, 14]),
            button("Reset")
                .on_press(Message::ResetDefaults)
                .padding([8, 14]),
        ]
        .spacing(10);

        if system_hotkeys::has_system_shortcut_configuration() {
            actions = actions.push(
                button("Configure Hotkeys")
                    .on_press(Message::ConfigureHotkeysRequested)
                    .padding([8, 14]),
            );
        }

        if scan::should_request_screen_capture_permission() {
            actions = actions.push(
                button("Reset Screen Token")
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
            button("Reward Scan").padding([8, 14])
        } else {
            button("Reward Scan")
                .on_press(Message::RewardScanRequested)
                .padding([8, 14])
        };
        let inventory_scan_button = if self.is_scanning {
            button("Inventory Scan").padding([8, 14])
        } else {
            button("Inventory Scan")
                .on_press(Message::InventoryScanRequested)
                .padding([8, 14])
        };

        let pipeline_actions = row![
            reward_scan_button,
            inventory_scan_button,
            button("Clear Market Cache")
                .on_press(Message::InvalidateMarketCacheRequested)
                .padding([8, 14]),
            button("Test Overlay")
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

    column![text("Results").size(22), body]
        .spacing(8)
        .height(Length::Fill)
        .into()
}

fn scan_results_body(app: &SettingsApp) -> Element<'_, Message> {
    let Some(output) = app.last_scan.as_ref() else {
        return centered_results_text("No scan results yet");
    };

    let items = reward_cards_from_scan_output(output);

    if items.is_empty() {
        return centered_results_text("No items found");
    }

    let cards = ui_core::reward_cards_row(items, &app.reward_card_assets)
        .wrap()
        .vertical_spacing(ui_core::REWARD_CARD_SPACING)
        .align_x(alignment::Horizontal::Center);

    let centered_cards = container(cards).center(Length::Fill);

    scrollable(centered_cards)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn centered_results_text(label: &'static str) -> Element<'static, Message> {
    container(text(label).size(14)).center(Length::Fill).into()
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

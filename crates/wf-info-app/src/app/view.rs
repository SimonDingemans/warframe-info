use iced::{
    alignment,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
    Element, Length,
};
use wf_info_core::ScanOutput;

use crate::{hotkeys, scan};

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

        let mut actions = row![
            button("Save").on_press(Message::Save).padding([8, 14]),
            button("Reset")
                .on_press(Message::ResetDefaults)
                .padding([8, 14]),
        ]
        .spacing(10);

        if hotkeys::has_system_shortcut_configuration() {
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

        column![hotkeys, actions, text(&self.hotkey_status).size(14)]
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

        let results = scan_results(self.last_scan.as_ref());

        column![pipeline_actions, results].spacing(18).into()
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

fn scan_results(output: Option<&ScanOutput>) -> Element<'_, Message> {
    let mut content = column![text("Results").size(22)].spacing(8);

    let Some(output) = output else {
        return content.push(text("No scan results yet").size(14)).into();
    };

    content = content.push(
        text(format!(
            "{} scan, source {}x{}, crop {}x{}",
            output.kind.label(),
            output.source_width,
            output.source_height,
            output.cropped_width,
            output.cropped_height,
        ))
        .size(14),
    );

    if output.items.is_empty() {
        content = content.push(text("No items found").size(14));
    } else {
        for item in &output.items {
            content = content.push(text(item.summary()).size(16));
        }
    }

    scrollable(content).height(Length::Fill).into()
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

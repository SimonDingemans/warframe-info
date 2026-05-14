use iced::{
    alignment,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
    Element, Length,
};
use wf_info_core::ScanOutput;

use crate::hotkeys;

use super::{message::Message, state::SettingsApp};

impl SettingsApp {
    pub(super) fn view(&self) -> Element<'_, Message> {
        let title = column![
            text("Warframe Info").size(30),
            text(self.settings_path.display().to_string()).size(14),
        ]
        .spacing(4);

        let hotkeys = column![
            text("Hotkeys").size(22),
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

        let pipeline_actions = row![reward_scan_button, inventory_scan_button].spacing(10);

        let results = scan_results(self.last_scan.as_ref());

        let status = if self.is_dirty {
            format!("{} - not saved", self.status)
        } else {
            self.status.clone()
        };

        container(
            column![
                title,
                rule::horizontal(1),
                hotkeys,
                actions,
                rule::horizontal(1),
                pipeline_actions,
                text(status).size(14),
                text(&self.hotkey_status).size(14),
                results,
            ]
            .spacing(18),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
            content = content.push(text(item).size(16));
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

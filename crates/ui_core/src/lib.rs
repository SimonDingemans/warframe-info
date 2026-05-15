pub const REWARD_CARD_WIDTH: u32 = 180;
pub const REWARD_CARD_HEIGHT: u32 = 154;
pub const REWARD_CARD_SPACING: u32 = 10;
pub const REWARD_OVERLAY_PADDING: u32 = 18;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardCardEntry {
    pub name: String,
    pub platinum: Option<u32>,
    pub ducats: Option<u32>,
    pub volume: Option<u32>,
    pub vaulted: bool,
    pub mastered: bool,
    pub owned_count: Option<u32>,
    pub required_count: Option<u32>,
    pub highlight: RewardHighlight,
}

impl RewardCardEntry {
    pub fn name_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            platinum: None,
            ducats: None,
            volume: None,
            vaulted: false,
            mastered: false,
            owned_count: None,
            required_count: None,
            highlight: RewardHighlight::None,
        }
    }

    pub fn with_platinum(mut self, platinum: u32) -> Self {
        self.platinum = Some(platinum);
        self
    }

    pub fn with_ducats(mut self, ducats: u32) -> Self {
        self.ducats = Some(ducats);
        self
    }

    pub fn with_volume(mut self, volume: u32) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn with_vaulted(mut self, vaulted: bool) -> Self {
        self.vaulted = vaulted;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RewardHighlight {
    #[default]
    None,
    BestPlatinum,
    BestDucats,
    Needed,
}

pub fn best_platinum_reward_index(rewards: &[RewardCardEntry]) -> Option<usize> {
    rewards
        .iter()
        .enumerate()
        .filter_map(|(index, reward)| reward.platinum.map(|platinum| (index, platinum)))
        .fold(None, |best, candidate| match best {
            Some((_, best_platinum)) if best_platinum >= candidate.1 => best,
            _ => Some(candidate),
        })
        .map(|(index, _)| index)
}

pub fn reward_is_best_platinum(
    index: usize,
    reward: &RewardCardEntry,
    best_platinum: Option<usize>,
) -> bool {
    reward.highlight == RewardHighlight::BestPlatinum || best_platinum == Some(index)
}

#[cfg(feature = "scan")]
pub fn reward_cards_from_scan_output(output: &info_core::ScanOutput) -> Vec<RewardCardEntry> {
    let limit = match output.kind {
        info_core::ScanKind::Reward => 4,
        info_core::ScanKind::Inventory => output.items.len(),
    };

    output
        .items
        .iter()
        .take(limit)
        .map(reward_card_from_item)
        .collect()
}

#[cfg(feature = "scan")]
pub fn reward_card_from_item(item: &info_core::WarframeItem) -> RewardCardEntry {
    RewardCardEntry {
        name: item.drop_name.clone(),
        platinum: Some(item.platinum_rounded()),
        ducats: item.ducats,
        volume: Some(item.volume),
        vaulted: item.vaulted,
        mastered: false,
        owned_count: None,
        required_count: None,
        highlight: RewardHighlight::None,
    }
}

#[cfg(feature = "iced-ui")]
mod iced_ui {
    use iced::widget::image::Handle as ImageHandle;
    use iced::widget::{column, container, image as iced_image, row, text, Row};
    use iced::{Color, Element, Length, Renderer, Theme};

    use crate::{
        best_platinum_reward_index, reward_is_best_platinum, RewardCardEntry, REWARD_CARD_SPACING,
        REWARD_CARD_WIDTH,
    };

    pub const PLATINUM_ICON_BYTES: &[u8] = include_bytes!("../files/PlatinumLarge.png");
    pub const DUCAT_ICON_BYTES: &[u8] = include_bytes!("../files/OrokinDucats.png");

    #[derive(Clone, Debug)]
    pub struct RewardCardAssets {
        pub platinum_icon: ImageHandle,
        pub ducat_icon: ImageHandle,
    }

    impl RewardCardAssets {
        pub fn load() -> Self {
            Self {
                platinum_icon: reward_icon_handle(PLATINUM_ICON_BYTES, "PlatinumLarge.png"),
                ducat_icon: reward_icon_handle(DUCAT_ICON_BYTES, "OrokinDucats.png"),
            }
        }
    }

    pub fn reward_icon_handle(bytes: &[u8], asset_name: &str) -> ImageHandle {
        let icon = decode_reward_icon(bytes, asset_name);
        let (width, height) = icon.dimensions();

        ImageHandle::from_rgba(width, height, icon.into_raw())
    }

    pub fn decode_reward_icon(bytes: &[u8], asset_name: &str) -> image::RgbaImage {
        let icon = image::load_from_memory(bytes)
            .unwrap_or_else(|error| panic!("failed to decode reward asset {asset_name}: {error}"))
            .into_rgba8();

        trim_transparent_padding(icon)
    }

    fn trim_transparent_padding(icon: image::RgbaImage) -> image::RgbaImage {
        let (width, height) = icon.dimensions();
        let mut bounds = None::<(u32, u32, u32, u32)>;

        for y in 0..height {
            for x in 0..width {
                if icon.get_pixel(x, y)[3] == 0 {
                    continue;
                }

                bounds = Some(match bounds {
                    Some((min_x, min_y, max_x, max_y)) => {
                        (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                    }
                    None => (x, y, x, y),
                });
            }
        }

        let Some((min_x, min_y, max_x, max_y)) = bounds else {
            return icon;
        };

        image::imageops::crop_imm(&icon, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
            .to_image()
    }

    pub fn reward_cards_row<Message: 'static>(
        rewards: impl IntoIterator<Item = RewardCardEntry>,
        assets: &RewardCardAssets,
    ) -> Row<'static, Message, Theme, Renderer> {
        let rewards = rewards.into_iter().collect::<Vec<_>>();
        let best_platinum = best_platinum_reward_index(&rewards);

        rewards.into_iter().enumerate().fold(
            row![].spacing(REWARD_CARD_SPACING),
            |row, (index, reward)| {
                let is_best_platinum = reward_is_best_platinum(index, &reward, best_platinum);

                row.push(reward_card(reward, is_best_platinum, assets))
            },
        )
    }

    pub fn reward_card<Message: 'static>(
        reward: RewardCardEntry,
        is_best_platinum: bool,
        assets: &RewardCardAssets,
    ) -> Element<'static, Message, Theme, Renderer> {
        let details = column![
            text(reward.name).size(16).width(Length::Fill),
            reward_value_with_icon(reward.platinum, assets.platinum_icon.clone()),
            reward_value_with_icon(reward.ducats, assets.ducat_icon.clone()),
            reward_detail("Sold last 48 hours", reward.volume),
        ]
        .spacing(4);
        let details = if reward.vaulted {
            details.push(
                text("Vaulted")
                    .size(14)
                    .color(Color::from_rgb(0.85, 0.78, 0.56)),
            )
        } else {
            details
        };

        let border_color = if is_best_platinum {
            Color::from_rgb(1.0, 0.84, 0.0)
        } else {
            Color::from_rgb(0.30, 0.34, 0.38)
        };

        let border_width = if is_best_platinum { 3.0 } else { 1.0 };

        container(details)
            .padding(12)
            .width(Length::Fixed(REWARD_CARD_WIDTH as f32))
            .style(move |_theme| container::Style {
                background: Some(Color::from_rgba(0.08, 0.09, 0.11, 0.92).into()),
                border: iced::Border {
                    color: border_color,
                    width: border_width,
                    radius: 4.0.into(),
                },
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
            .into()
    }

    fn reward_detail<Message: 'static>(
        label: &'static str,
        value: Option<u32>,
    ) -> Element<'static, Message, Theme, Renderer> {
        text(format!(
            "{label}: {}",
            value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unknown".to_owned())
        ))
        .size(14)
        .into()
    }

    fn reward_value_with_icon<Message: 'static>(
        value: Option<u32>,
        icon: ImageHandle,
    ) -> Element<'static, Message, Theme, Renderer> {
        row![
            iced_image(icon)
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
            text(
                value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "Unknown".to_owned())
            )
            .size(14)
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            best_platinum_reward_index, reward_is_best_platinum, RewardCardEntry, RewardHighlight,
        };

        use super::{decode_reward_icon, DUCAT_ICON_BYTES, PLATINUM_ICON_BYTES};

        #[test]
        fn best_platinum_reward_index_uses_highest_available_platinum_value() {
            let rewards = vec![
                RewardCardEntry::name_only("Forma Blueprint").with_platinum(8),
                RewardCardEntry::name_only("Braton Prime Receiver").with_platinum(42),
                RewardCardEntry::name_only("Paris Prime String").with_platinum(15),
            ];

            assert_eq!(best_platinum_reward_index(&rewards), Some(1));
        }

        #[test]
        fn best_platinum_reward_index_ignores_missing_platinum_values() {
            let rewards = vec![
                RewardCardEntry::name_only("Forma Blueprint"),
                RewardCardEntry::name_only("Braton Prime Receiver").with_platinum(12),
            ];

            assert_eq!(best_platinum_reward_index(&rewards), Some(1));
        }

        #[test]
        fn reward_highlight_can_mark_best_platinum_without_price_data() {
            let mut reward = RewardCardEntry::name_only("Forma Blueprint");
            reward.highlight = RewardHighlight::BestPlatinum;

            assert!(reward_is_best_platinum(0, &reward, None));
        }

        #[test]
        fn reward_icon_assets_decode_and_trim_transparent_padding() {
            let platinum_icon = decode_reward_icon(PLATINUM_ICON_BYTES, "PlatinumLarge.png");
            let ducat_icon = decode_reward_icon(DUCAT_ICON_BYTES, "OrokinDucats.png");

            assert_eq!(platinum_icon.dimensions(), (291, 285));
            assert_eq!(ducat_icon.dimensions(), (337, 430));
        }
    }
}

#[cfg(feature = "iced-ui")]
pub use iced_ui::{
    decode_reward_icon, reward_card, reward_cards_row, reward_icon_handle, RewardCardAssets,
    DUCAT_ICON_BYTES, PLATINUM_ICON_BYTES,
};

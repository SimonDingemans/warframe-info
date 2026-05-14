use iced::widget::image::Handle as ImageHandle;
use iced::widget::{button, column, container, image as iced_image, row, rule, text};
use iced::{time, Color, Element, Font, Length, Pixels, Renderer, Subscription, Task, Theme};
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use std::time::Duration;

mod display;
pub mod platform;

pub use display::{
    display_outputs, reset_display_restore_token, DisplayBackend, DisplayOutput,
    DisplayOutputsFuture, DisplayResult, DynDisplayBackend,
};

const DEFAULT_MONITOR_WIDTH: u32 = 1920;
const DEFAULT_MONITOR_HEIGHT: u32 = 1080;
const REWARD_CARD_WIDTH: u32 = 180;
const REWARD_CARD_HEIGHT: u32 = 154;
const REWARD_CARD_SPACING: u32 = 10;
const REWARD_OVERLAY_PADDING: u32 = 18;
const REWARD_OVERLAY_Y_RATIO: f32 = 0.62;
const DEFAULT_OVERLAY_DURATION: Duration = Duration::from_secs(12);
const PLATINUM_ICON_BYTES: &[u8] = include_bytes!("../assets/src/PlatinumLarge.png");
const DUCAT_ICON_BYTES: &[u8] = include_bytes!("../assets/src/OrokinDucats.png");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardOverlayEntry {
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

impl RewardOverlayEntry {
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

#[derive(Clone, Debug)]
pub struct RewardOverlay {
    pub output_name: Option<String>,
    pub output_size: Option<(u32, u32)>,
    pub duration: Option<Duration>,
    pub rewards: Vec<RewardOverlayEntry>,
}

pub fn run(overlay: RewardOverlay) -> iced_layershell::Result {
    let start_mode = overlay
        .output_name
        .as_ref()
        .map(|output| StartMode::TargetScreen(output.to_owned()))
        .unwrap_or(StartMode::Active);
    let layer_settings = layer_settings_for_overlay(&overlay, start_mode);

    iced_layershell::disable_clipboard();

    application(
        move || RewardOverlayApp {
            overlay: overlay.clone(),
            assets: RewardOverlayAssets::load(),
        },
        namespace,
        update,
        view,
    )
    .theme(Theme::Dark)
    .style(style)
    .subscription(subscription)
    .settings(Settings {
        id: Some("wf-info.reward-overlay".to_owned()),
        layer_settings,
        fonts: Vec::new(),
        default_font: Font::default(),
        default_text_size: Pixels(16.0),
        antialiasing: true,
        virtual_keyboard_support: None,
        with_connection: None,
    })
    .run()
}

fn layer_settings_for_overlay(
    overlay: &RewardOverlay,
    start_mode: StartMode,
) -> LayerShellSettings {
    let placement = reward_overlay_placement(overlay.output_size, overlay.rewards.len());

    LayerShellSettings {
        anchor: Anchor::Top | Anchor::Left,
        layer: Layer::Overlay,
        exclusive_zone: -1,
        size: Some(placement.size),
        margin: placement.margin,
        keyboard_interactivity: KeyboardInteractivity::None,
        events_transparent: false,
        start_mode,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RewardOverlayPlacement {
    margin: (i32, i32, i32, i32),
    size: (u32, u32),
}

fn reward_overlay_placement(
    output_size: Option<(u32, u32)>,
    reward_count: usize,
) -> RewardOverlayPlacement {
    let (monitor_width, monitor_height) =
        output_size.unwrap_or((DEFAULT_MONITOR_WIDTH, DEFAULT_MONITOR_HEIGHT));
    let size = reward_overlay_surface_size(reward_count, monitor_width);
    let top = (monitor_height as f32 * REWARD_OVERLAY_Y_RATIO).round() as i32;
    let left = ((monitor_width.saturating_sub(size.0)) / 2) as i32;

    RewardOverlayPlacement {
        margin: (top, 0, 0, left),
        size,
    }
}

fn reward_overlay_surface_size(reward_count: usize, monitor_width: u32) -> (u32, u32) {
    let reward_count = reward_count.clamp(1, 4) as u32;
    let card_width = reward_count * REWARD_CARD_WIDTH;
    let spacing = reward_count.saturating_sub(1) * REWARD_CARD_SPACING;
    let width = (card_width + spacing + 2 * REWARD_OVERLAY_PADDING).min(monitor_width);
    let height = REWARD_CARD_HEIGHT + 2 * REWARD_OVERLAY_PADDING;

    (width, height)
}

struct RewardOverlayApp {
    overlay: RewardOverlay,
    assets: RewardOverlayAssets,
}

#[derive(Clone, Debug)]
struct RewardOverlayAssets {
    platinum_icon: ImageHandle,
    ducat_icon: ImageHandle,
}

impl RewardOverlayAssets {
    fn load() -> Self {
        Self {
            platinum_icon: reward_icon_handle(PLATINUM_ICON_BYTES, "PlatinumLarge.png"),
            ducat_icon: reward_icon_handle(DUCAT_ICON_BYTES, "OrokinDucats.png"),
        }
    }
}

fn reward_icon_handle(bytes: &[u8], asset_name: &str) -> ImageHandle {
    let icon = decode_reward_icon(bytes, asset_name);
    let (width, height) = icon.dimensions();

    ImageHandle::from_rgba(width, height, icon.into_raw())
}

fn decode_reward_icon(bytes: &[u8], asset_name: &str) -> image::RgbaImage {
    let icon = image::load_from_memory(bytes)
        .unwrap_or_else(|error| panic!("failed to decode overlay asset {asset_name}: {error}"))
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

    image::imageops::crop_imm(&icon, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image()
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Close,
    Tick,
}

fn namespace() -> String {
    "wf-info reward overlay".to_owned()
}

fn update(_app: &mut RewardOverlayApp, message: Message) -> Task<Message> {
    match message {
        Message::Close | Message::Tick => std::process::exit(0),
        _ => Task::none(),
    }
}

fn subscription(app: &RewardOverlayApp) -> Subscription<Message> {
    time::every(app.overlay.duration.unwrap_or(DEFAULT_OVERLAY_DURATION)).map(|_| Message::Tick)
}

fn view(app: &RewardOverlayApp) -> Element<'_, Message, Theme, Renderer> {
    reward_overlay_view(&app.overlay.rewards, &app.assets)
}

fn style(_app: &RewardOverlayApp, _theme: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    }
}

fn reward_overlay_view<'a>(
    rewards: &'a [RewardOverlayEntry],
    assets: &'a RewardOverlayAssets,
) -> Element<'a, Message, Theme, Renderer> {
    let best_platinum = best_platinum_reward_index(rewards);
    let reward_cards =
        rewards
            .iter()
            .enumerate()
            .fold(row![].spacing(10), |row, (index, reward)| {
                row.push(reward_card(
                    reward,
                    reward_is_best_platinum(index, reward, best_platinum),
                    assets,
                ))
            });

    container(reward_cards)
        .padding(18)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgba(0.03, 0.04, 0.05, 0.78).into()),
            border: iced::Border {
                color: Color::from_rgb(0.20, 0.23, 0.27),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        })
        .into()
}

fn reward_card<'a>(
    reward: &'a RewardOverlayEntry,
    is_best_platinum: bool,
    assets: &'a RewardOverlayAssets,
) -> Element<'a, Message, Theme, Renderer> {
    let details = column![
        text(&reward.name).size(16).width(Length::Fill),
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
        .width(Length::Fixed(180.0))
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

fn reward_detail(
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

fn reward_value_with_icon(
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

fn reward_is_best_platinum(
    index: usize,
    reward: &RewardOverlayEntry,
    best_platinum: Option<usize>,
) -> bool {
    reward.highlight == RewardHighlight::BestPlatinum || best_platinum == Some(index)
}

fn best_platinum_reward_index(rewards: &[RewardOverlayEntry]) -> Option<usize> {
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

#[allow(dead_code)]
fn monitor_info_view(lines: &[String]) -> Element<'_, Message, Theme, Renderer> {
    let details = lines.iter().fold(column![].spacing(4), |column, line| {
        column.push(text(line).size(18))
    });

    container(
        column![
            text("wf-info monitor debug").size(24),
            rule::horizontal(1),
            details,
            button("Quit Overlay")
                .padding([8, 12])
                .on_press(Message::Close),
        ]
        .spacing(8),
    )
    .padding(18)
    .width(Length::Shrink)
    .height(Length::Shrink)
    .style(|_theme| container::Style {
        background: Some(Color::from_rgba(0.04, 0.05, 0.07, 0.82).into()),
        border: iced::Border {
            color: Color::from_rgb(0.22, 0.78, 0.72),
            width: 2.0,
            radius: 4.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::{
        best_platinum_reward_index, decode_reward_icon, reward_is_best_platinum,
        reward_overlay_placement, reward_overlay_surface_size, RewardHighlight, RewardOverlayEntry,
        DEFAULT_MONITOR_HEIGHT, DEFAULT_MONITOR_WIDTH, DUCAT_ICON_BYTES, PLATINUM_ICON_BYTES,
        REWARD_CARD_HEIGHT, REWARD_CARD_SPACING, REWARD_CARD_WIDTH, REWARD_OVERLAY_PADDING,
    };

    #[test]
    fn best_platinum_reward_index_uses_highest_available_platinum_value() {
        let rewards = vec![
            RewardOverlayEntry::name_only("Forma Blueprint").with_platinum(8),
            RewardOverlayEntry::name_only("Braton Prime Receiver").with_platinum(42),
            RewardOverlayEntry::name_only("Paris Prime String").with_platinum(15),
        ];

        assert_eq!(best_platinum_reward_index(&rewards), Some(1));
    }

    #[test]
    fn best_platinum_reward_index_ignores_missing_platinum_values() {
        let rewards = vec![
            RewardOverlayEntry::name_only("Forma Blueprint"),
            RewardOverlayEntry::name_only("Braton Prime Receiver").with_platinum(12),
        ];

        assert_eq!(best_platinum_reward_index(&rewards), Some(1));
    }

    #[test]
    fn reward_highlight_can_mark_best_platinum_without_price_data() {
        let mut reward = RewardOverlayEntry::name_only("Forma Blueprint");
        reward.highlight = RewardHighlight::BestPlatinum;

        assert!(reward_is_best_platinum(0, &reward, None));
    }

    #[test]
    fn reward_overlay_surface_size_tracks_card_count() {
        let expected_width =
            4 * REWARD_CARD_WIDTH + 3 * REWARD_CARD_SPACING + 2 * REWARD_OVERLAY_PADDING;
        let expected_height = REWARD_CARD_HEIGHT + 2 * REWARD_OVERLAY_PADDING;

        assert_eq!(
            reward_overlay_surface_size(4, DEFAULT_MONITOR_WIDTH),
            (expected_width, expected_height)
        );
    }

    #[test]
    fn reward_overlay_surface_size_clamps_to_monitor_width() {
        assert_eq!(reward_overlay_surface_size(4, 320).0, 320);
    }

    #[test]
    fn reward_overlay_placement_uses_default_monitor_when_output_size_is_unknown() {
        let placement = reward_overlay_placement(None, 4);
        let expected_size = reward_overlay_surface_size(4, DEFAULT_MONITOR_WIDTH);

        assert_eq!(placement.size, expected_size);
        assert_eq!(
            placement.margin,
            (
                (DEFAULT_MONITOR_HEIGHT as f32 * super::REWARD_OVERLAY_Y_RATIO).round() as i32,
                0,
                0,
                ((DEFAULT_MONITOR_WIDTH - expected_size.0) / 2) as i32
            )
        );
    }

    #[test]
    fn reward_icon_assets_decode_and_trim_transparent_padding() {
        let platinum_icon = decode_reward_icon(PLATINUM_ICON_BYTES, "PlatinumLarge.png");
        let ducat_icon = decode_reward_icon(DUCAT_ICON_BYTES, "OrokinDucats.png");

        assert_eq!(platinum_icon.dimensions(), (291, 285));
        assert_eq!(ducat_icon.dimensions(), (337, 430));
    }
}

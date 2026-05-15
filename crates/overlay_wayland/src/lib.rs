use iced::widget::{button, column, container, rule, text};
use iced::{time, Color, Element, Font, Length, Pixels, Renderer, Subscription, Task, Theme};
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use overlay::{DisplayBackend, DisplayOutput, RewardOverlay};
use std::time::Duration;
use ui_core::RewardCardAssets;

mod display;

pub use display::LinuxWaylandDisplayBackend;

const DEFAULT_MONITOR_WIDTH: u32 = 1920;
const DEFAULT_MONITOR_HEIGHT: u32 = 1080;
const REWARD_OVERLAY_Y_RATIO: f32 = 0.62;
const DEFAULT_OVERLAY_DURATION: Duration = Duration::from_secs(12);

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
            assets: RewardCardAssets::load(),
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

pub async fn display_outputs() -> overlay::DisplayResult<Vec<DisplayOutput>> {
    LinuxWaylandDisplayBackend::new().display_outputs().await
}

pub fn reset_display_restore_token() -> overlay::DisplayResult<()> {
    display::reset_screencast_token()
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
    let card_width = reward_count * ui_core::REWARD_CARD_WIDTH;
    let spacing = reward_count.saturating_sub(1) * ui_core::REWARD_CARD_SPACING;
    let width = (card_width + spacing + 2 * ui_core::REWARD_OVERLAY_PADDING).min(monitor_width);
    let height = ui_core::REWARD_CARD_HEIGHT + 2 * ui_core::REWARD_OVERLAY_PADDING;

    (width, height)
}

struct RewardOverlayApp {
    overlay: RewardOverlay,
    assets: RewardCardAssets,
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
    rewards: &'a [ui_core::RewardCardEntry],
    assets: &'a RewardCardAssets,
) -> Element<'a, Message, Theme, Renderer> {
    let reward_cards = ui_core::reward_cards_row(rewards.iter().cloned(), assets);

    container(reward_cards)
        .padding(ui_core::REWARD_OVERLAY_PADDING as u16)
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
        reward_overlay_placement, reward_overlay_surface_size, DEFAULT_MONITOR_HEIGHT,
        DEFAULT_MONITOR_WIDTH,
    };

    #[test]
    fn reward_overlay_surface_size_tracks_card_count() {
        let expected_width = 4 * ui_core::REWARD_CARD_WIDTH
            + 3 * ui_core::REWARD_CARD_SPACING
            + 2 * ui_core::REWARD_OVERLAY_PADDING;
        let expected_height = ui_core::REWARD_CARD_HEIGHT + 2 * ui_core::REWARD_OVERLAY_PADDING;

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
}

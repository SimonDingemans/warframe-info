use std::path::PathBuf;

use iced::{window, Size, Theme};

mod message;
mod state;
mod update;
mod view;

use state::SettingsApp;

pub(crate) fn run(settings_path: PathBuf) -> Result<(), String> {
    iced::application(
        move || SettingsApp::load(settings_path.clone()),
        SettingsApp::update,
        SettingsApp::view,
    )
    .title("Warframe Info")
    .theme(Theme::Dark)
    .subscription(SettingsApp::subscription)
    .window(window::Settings {
        size: Size::new(1280.0, 720.0),
        min_size: Some(Size::new(520.0, 360.0)),
        ..Default::default()
    })
    .run()
    .map_err(|error| error.to_string())
}

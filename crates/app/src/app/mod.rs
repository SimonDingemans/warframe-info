use std::path::PathBuf;

use iced::{window, Size, Task, Theme};

mod message;
mod state;
mod update;
mod view;

use state::SettingsApp;

pub(crate) fn run(settings_path: PathBuf) -> Result<(), String> {
    iced::application(
        move || {
            let mut app = SettingsApp::load(settings_path.clone());

            if crate::scan::should_request_screen_capture_permission() {
                app.status = "Requesting screen capture permission".to_owned();

                (
                    app,
                    Task::perform(crate::scan::request_screen_capture_permission(), |result| {
                        message::Message::ScreenCapturePermissionFinished(result)
                    }),
                )
            } else {
                (app, Task::none())
            }
        },
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

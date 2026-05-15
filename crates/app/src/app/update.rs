use iced::Task;
use info_core::{AppSettings, ScanKind};

use hotkeys::HotkeyEvent;

use crate::{
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
        }

        Task::none()
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

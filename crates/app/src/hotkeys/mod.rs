use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod platform;

use global_hotkey::{
    hotkey::{HotKey, HotKeyParseError},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use iced::{
    futures::{channel::mpsc, executor, future, SinkExt, Stream},
    stream, Subscription,
};
use info_core::{platform::GlobalShortcutBackend, AppSettings, ScanKind};

pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub(super) trait SystemShortcutIntegration: Sync {
    fn registration_status(&self, settings: &AppSettings) -> String;

    fn configure_shortcuts(&self, settings: AppSettings) -> BoxFuture<Result<String, String>>;

    fn watch_shortcuts(
        &self,
        settings: AppSettings,
        sender: mpsc::Sender<HotkeyEvent>,
    ) -> BoxFuture<()>;
}

pub(crate) fn has_system_shortcut_configuration() -> bool {
    info_core::platform::current().global_shortcut_backend()
        == GlobalShortcutBackend::SystemConfigured
}

pub(crate) fn configure_system_shortcuts(
    settings: AppSettings,
) -> BoxFuture<Result<String, String>> {
    platform::system_shortcuts().configure_shortcuts(settings)
}

pub(crate) struct HotkeyBindings {
    backend: HotkeyBackendState,
}

impl HotkeyBindings {
    pub(crate) fn new(settings: &AppSettings) -> (Self, String) {
        let mut bindings = Self {
            backend: make_hotkey_backend(),
        };
        let status = bindings.configure(settings);
        (bindings, status)
    }

    pub(crate) fn configure(&mut self, settings: &AppSettings) -> String {
        self.backend.configure(settings)
    }

    pub(crate) fn subscription(&self, settings: &AppSettings) -> Subscription<HotkeyEvent> {
        Subscription::run_with(
            self.backend.watcher_config(settings),
            hotkey_event_subscription,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HotkeyEvent {
    Triggered(ScanKind),
    Status(String),
}

enum HotkeyBackendState {
    Native(NativeHotkeyBackend),
    SystemConfigured,
}

impl HotkeyBackendState {
    fn configure(&mut self, settings: &AppSettings) -> String {
        match self {
            Self::Native(backend) => backend.configure(settings),
            Self::SystemConfigured => platform::system_shortcuts().registration_status(settings),
        }
    }

    fn watcher_config(&self, settings: &AppSettings) -> HotkeyWatcherConfig {
        match self {
            Self::Native(backend) => HotkeyWatcherConfig::Native(backend.registered_bindings()),
            Self::SystemConfigured => HotkeyWatcherConfig::SystemConfigured {
                reward_scan: settings.hotkeys.reward_scan.clone(),
                inventory_scan: settings.hotkeys.inventory_scan.clone(),
            },
        }
    }
}

fn make_hotkey_backend() -> HotkeyBackendState {
    if has_system_shortcut_configuration() {
        return HotkeyBackendState::SystemConfigured;
    }

    HotkeyBackendState::Native(NativeHotkeyBackend::new())
}

struct NativeHotkeyBackend {
    manager: Option<GlobalHotKeyManager>,
    registered: Vec<HotKey>,
    ids: HashMap<u32, ScanKind>,
    startup_error: Option<String>,
}

impl NativeHotkeyBackend {
    fn new() -> Self {
        match GlobalHotKeyManager::new() {
            Ok(manager) => Self {
                manager: Some(manager),
                registered: Vec::new(),
                ids: HashMap::new(),
                startup_error: None,
            },
            Err(error) => Self {
                manager: None,
                registered: Vec::new(),
                ids: HashMap::new(),
                startup_error: Some(error.to_string()),
            },
        }
    }

    fn unregister_all(&mut self) {
        let Some(manager) = &self.manager else {
            return;
        };

        for hotkey in self.registered.drain(..) {
            let _ = manager.unregister(hotkey);
        }
    }
}

impl NativeHotkeyBackend {
    fn configure(&mut self, settings: &AppSettings) -> String {
        self.unregister_all();
        self.ids.clear();

        let Some(manager) = &self.manager else {
            return format!(
                "Global hotkeys unavailable: {}",
                self.startup_error
                    .as_deref()
                    .unwrap_or("native backend could not start")
            );
        };

        let mut messages = Vec::new();
        let registrations = [
            (
                ScanKind::Reward,
                "reward scan",
                settings.hotkeys.reward_scan.as_str(),
            ),
            (
                ScanKind::Inventory,
                "inventory scan",
                settings.hotkeys.inventory_scan.as_str(),
            ),
        ];

        for (kind, label, hotkey) in registrations {
            match parse_native_hotkey(hotkey) {
                Ok(parsed) => match manager.register(parsed) {
                    Ok(()) => {
                        self.ids.insert(parsed.id(), kind);
                        self.registered.push(parsed);
                    }
                    Err(error) => messages.push(format!(
                        "Could not register {label} hotkey `{hotkey}`: {error}"
                    )),
                },
                Err(error) => {
                    messages.push(format!("Invalid {label} hotkey `{hotkey}`: {error}"));
                }
            }
        }

        if messages.is_empty() {
            "Global hotkeys registered with native backend".to_owned()
        } else {
            messages.join("; ")
        }
    }

    fn registered_bindings(&self) -> Vec<NativeHotkeyBinding> {
        let mut bindings: Vec<_> = self
            .ids
            .iter()
            .map(|(id, kind)| NativeHotkeyBinding {
                id: *id,
                kind: *kind,
            })
            .collect();

        bindings.sort_by_key(|binding| binding.id);
        bindings
    }
}

fn parse_native_hotkey(hotkey: &str) -> Result<HotKey, HotKeyParseError> {
    hotkey.trim().parse()
}

#[derive(Debug, Clone, Hash)]
enum HotkeyWatcherConfig {
    Native(Vec<NativeHotkeyBinding>),
    SystemConfigured {
        reward_scan: String,
        inventory_scan: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NativeHotkeyBinding {
    id: u32,
    kind: ScanKind,
}

fn hotkey_event_subscription(config: &HotkeyWatcherConfig) -> impl Stream<Item = HotkeyEvent> {
    let config = config.clone();

    stream::channel(32, move |sender| async move {
        match config {
            HotkeyWatcherConfig::Native(bindings) => {
                watch_native_hotkeys(bindings, sender).await;
            }
            HotkeyWatcherConfig::SystemConfigured {
                reward_scan,
                inventory_scan,
            } => {
                let settings = AppSettings {
                    hotkeys: info_core::HotkeySettings {
                        reward_scan,
                        inventory_scan,
                    },
                };

                platform::system_shortcuts()
                    .watch_shortcuts(settings, sender)
                    .await;
            }
        }
    })
}

async fn watch_native_hotkeys(
    bindings: Vec<NativeHotkeyBinding>,
    mut sender: mpsc::Sender<HotkeyEvent>,
) {
    if bindings.is_empty() {
        let _ = sender
            .send(HotkeyEvent::Status(
                "Native global hotkey watcher has no registered hotkeys".to_owned(),
            ))
            .await;
        future::pending::<()>().await;
    }

    let ids: HashMap<u32, ScanKind> = bindings
        .into_iter()
        .map(|binding| (binding.id, binding.kind))
        .collect();
    let running = Arc::new(AtomicBool::new(true));
    let guard = HotkeyWatcherGuard {
        running: Arc::clone(&running),
    };

    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();

        while running.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(event) if event.state() == HotKeyState::Pressed => {
                    let Some(kind) = ids.get(&event.id()).copied() else {
                        continue;
                    };

                    if executor::block_on(sender.send(HotkeyEvent::Triggered(kind))).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    });

    future::pending::<()>().await;
    drop(guard);
}

struct HotkeyWatcherGuard {
    running: Arc<AtomicBool>,
}

impl Drop for HotkeyWatcherGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

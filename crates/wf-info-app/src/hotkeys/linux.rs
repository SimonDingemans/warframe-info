use std::{collections::HashMap, env, process::Command as ProcessCommand};

use ashpd::desktop::{
    global_shortcuts::{
        BindShortcutsOptions, ConfigureShortcutsOptions, GlobalShortcuts, ListShortcutsOptions,
        NewShortcut, Shortcut,
    },
    CreateSessionOptions,
};
use ashpd::WindowIdentifier;
use futures_util::StreamExt;
use iced::futures::{channel::mpsc, SinkExt};
use wf_info_core::{AppSettings, ScanKind};

use super::super::{BoxFuture, HotkeyEvent, SystemShortcutIntegration};

pub(super) static SYSTEM_SHORTCUTS: LinuxSystemShortcuts = LinuxSystemShortcuts;

pub(super) struct LinuxSystemShortcuts;

impl SystemShortcutIntegration for LinuxSystemShortcuts {
    fn registration_status(&self, _settings: &AppSettings) -> String {
        "Wayland global shortcuts requested through the desktop portal".to_owned()
    }

    fn configure_shortcuts(&self, settings: AppSettings) -> BoxFuture<Result<String, String>> {
        Box::pin(configure_desktop_portal_shortcuts(settings))
    }

    fn watch_shortcuts(
        &self,
        settings: AppSettings,
        sender: mpsc::Sender<HotkeyEvent>,
    ) -> BoxFuture<()> {
        Box::pin(watch_wayland_portal_hotkeys(settings, sender))
    }
}

async fn configure_desktop_portal_shortcuts(settings: AppSettings) -> Result<String, String> {
    let portal = GlobalShortcuts::new()
        .await
        .map_err(|error| format!("Wayland global shortcuts unavailable: {error}"))?;
    let portal_version = portal.version();
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await
        .map_err(|error| format!("could not create global shortcut session: {error}"))?;
    let shortcuts = wayland_shortcut_specs(&settings);
    let new_shortcuts = wayland_new_shortcuts(&shortcuts);

    let bind_request = portal
        .bind_shortcuts(
            &session,
            &new_shortcuts,
            Option::<&WindowIdentifier>::None,
            BindShortcutsOptions::default(),
        )
        .await
        .map_err(|error| format!("could not register shortcuts with desktop portal: {error}"))?;
    let bind_response = bind_request
        .response()
        .map_err(|error| format!("shortcut registration was not completed: {error}"))?;

    let configure_status = if portal_version >= 2 {
        match portal
            .configure_shortcuts(
                &session,
                Option::<&WindowIdentifier>::None,
                ConfigureShortcutsOptions::default(),
            )
            .await
        {
            Ok(()) => "Desktop shortcut configuration opened.".to_owned(),
            Err(error) => format!("Could not open desktop shortcut configuration: {error}."),
        }
    } else {
        let settings_status = open_kde_shortcut_settings()
            .map(|()| " Opened KDE Shortcuts settings.".to_owned())
            .unwrap_or_else(|error| format!(" {error}"));

        format!(
            "Desktop shortcut configuration requires portal v2; this desktop exposes v{portal_version}.{settings_status} Assign the listed actions manually in KDE Shortcuts."
        )
    };

    let list_request = portal
        .list_shortcuts(&session, ListShortcutsOptions::default())
        .await
        .map_err(|error| format!("could not list registered desktop shortcuts: {error}"))?;
    let list_response = list_request
        .response()
        .map_err(|error| format!("desktop shortcut list was not returned: {error}"))?;

    Ok(format!(
        "Wayland shortcuts registered with desktop portal ({}/{} accepted). {} {}",
        bind_response.shortcuts().len(),
        shortcuts.len(),
        configure_status,
        wayland_shortcut_summary(list_response.shortcuts(), &shortcuts),
    ))
}

async fn watch_wayland_portal_hotkeys(settings: AppSettings, sender: mpsc::Sender<HotkeyEvent>) {
    watch_shortcuts(wayland_shortcut_specs(&settings), sender).await;
}

fn open_kde_shortcut_settings() -> Result<(), String> {
    if !is_kde_plasma_session() {
        return Err(
            "Open your desktop's shortcut settings to assign the listed actions.".to_owned(),
        );
    }

    let candidates = [
        ("systemsettings", "kcm_keys"),
        ("kcmshell6", "kcm_keys"),
        ("kcmshell5", "keys"),
    ];

    for (program, module) in candidates {
        if ProcessCommand::new(program).arg(module).spawn().is_ok() {
            return Ok(());
        }
    }

    Err("Open KDE System Settings > Keyboard > Shortcuts to assign the listed actions.".to_owned())
}

fn is_kde_plasma_session() -> bool {
    env::var("XDG_CURRENT_DESKTOP")
        .map(|desktop| {
            desktop
                .split(':')
                .any(|part| part.eq_ignore_ascii_case("KDE"))
        })
        .unwrap_or(false)
        || env::var("KDE_FULL_SESSION")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        || env::var("DESKTOP_SESSION")
            .map(|value| value.to_ascii_lowercase().contains("plasma"))
            .unwrap_or(false)
}

struct WaylandShortcutSpec {
    id: &'static str,
    kind: ScanKind,
    description: &'static str,
    preferred_trigger: String,
}

fn wayland_shortcut_specs(settings: &AppSettings) -> Vec<WaylandShortcutSpec> {
    vec![
        WaylandShortcutSpec {
            id: "reward_scan",
            kind: ScanKind::Reward,
            description: "Scan Warframe reward choices",
            preferred_trigger: xdg_shortcut_trigger(&settings.hotkeys.reward_scan),
        },
        WaylandShortcutSpec {
            id: "inventory_scan",
            kind: ScanKind::Inventory,
            description: "Scan Warframe inventory",
            preferred_trigger: xdg_shortcut_trigger(&settings.hotkeys.inventory_scan),
        },
    ]
}

fn wayland_new_shortcuts(shortcuts: &[WaylandShortcutSpec]) -> Vec<NewShortcut> {
    shortcuts
        .iter()
        .map(|shortcut| {
            NewShortcut::new(shortcut.id, shortcut.description)
                .preferred_trigger(Some(shortcut.preferred_trigger.as_str()))
        })
        .collect()
}

fn wayland_shortcut_summary(shortcuts: &[Shortcut], desired: &[WaylandShortcutSpec]) -> String {
    if shortcuts.is_empty() {
        return "No desktop shortcuts are currently bound.".to_owned();
    }

    let desired: HashMap<_, _> = desired
        .iter()
        .map(|shortcut| (shortcut.id, shortcut.preferred_trigger.as_str()))
        .collect();

    shortcuts
        .iter()
        .map(|shortcut| {
            let trigger = shortcut.trigger_description().trim();

            if trigger.is_empty() {
                if let Some(preferred_trigger) = desired.get(shortcut.id()) {
                    format!(
                        "{}: unassigned, requested {preferred_trigger}",
                        shortcut.description()
                    )
                } else {
                    format!("{}: unassigned", shortcut.description())
                }
            } else {
                format!("{}: {trigger}", shortcut.description())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

async fn watch_shortcuts(
    shortcuts: Vec<WaylandShortcutSpec>,
    mut sender: mpsc::Sender<HotkeyEvent>,
) {
    let portal = match GlobalShortcuts::new().await {
        Ok(portal) => portal,
        Err(error) => {
            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Wayland global shortcuts unavailable: {error}"
                )))
                .await;
            return;
        }
    };

    let session = match portal.create_session(CreateSessionOptions::default()).await {
        Ok(session) => session,
        Err(error) => {
            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Could not create Wayland global shortcut session: {error}"
                )))
                .await;
            return;
        }
    };

    let new_shortcuts = wayland_new_shortcuts(&shortcuts);

    let request = match portal
        .bind_shortcuts(
            &session,
            &new_shortcuts,
            Option::<&WindowIdentifier>::None,
            BindShortcutsOptions::default(),
        )
        .await
    {
        Ok(request) => request,
        Err(error) => {
            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Could not request Wayland global shortcuts: {error}"
                )))
                .await;
            return;
        }
    };

    match request.response() {
        Ok(response) => {
            let count = response.shortcuts().len();
            let list_status = match portal
                .list_shortcuts(&session, ListShortcutsOptions::default())
                .await
            {
                Ok(request) => match request.response() {
                    Ok(response) => {
                        format!(
                            " {}",
                            wayland_shortcut_summary(response.shortcuts(), &shortcuts)
                        )
                    }
                    Err(error) => format!(" Could not read desktop shortcut list: {error}"),
                },
                Err(error) => format!(" Could not request desktop shortcut list: {error}"),
            };

            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Wayland global shortcuts registered through desktop portal ({count}/{} accepted).{list_status}",
                    shortcuts.len(),
                )))
                .await;
        }
        Err(error) => {
            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Wayland global shortcut binding was not completed: {error}"
                )))
                .await;
            return;
        }
    }

    let mut activated = match portal.receive_activated().await {
        Ok(activated) => Box::pin(activated),
        Err(error) => {
            let _ = sender
                .send(HotkeyEvent::Status(format!(
                    "Could not listen for Wayland global shortcuts: {error}"
                )))
                .await;
            return;
        }
    };

    loop {
        let Some(event) = activated.next().await else {
            break;
        };

        if let Some(shortcut) = shortcuts
            .iter()
            .find(|shortcut| shortcut.id == event.shortcut_id())
        {
            let _ = sender.send(HotkeyEvent::Triggered(shortcut.kind)).await;
        }
    }
}

fn xdg_shortcut_trigger(hotkey: &str) -> String {
    hotkey
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => "CTRL".to_owned(),
            "alt" | "option" => "ALT".to_owned(),
            "shift" => "SHIFT".to_owned(),
            "super" | "cmd" | "command" | "logo" => "LOGO".to_owned(),
            key => xdg_key_name(key),
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn xdg_key_name(key: &str) -> String {
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(single), None) if single.is_ascii_alphabetic() => {
            single.to_ascii_lowercase().to_string()
        }
        _ => key.to_owned(),
    }
}

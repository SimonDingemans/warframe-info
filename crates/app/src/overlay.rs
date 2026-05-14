use std::{
    ffi::OsString,
    process::{Command, Stdio},
    time::Duration,
};

use info_core::{ScanKind, ScanOutput, WarframeItem};
use overlay::{RewardHighlight, RewardOverlay, RewardOverlayEntry};

const REWARD_OVERLAY_ARG: &str = "--wf-info-reward-overlay";
const TEST_REWARD_OVERLAY_ARG: &str = "--wf-info-test-reward-overlay";
const OUTPUT_SIZE_ARG: &str = "--output-size";
const TARGET_OUTPUT_ARG: &str = "--target-output";
const TARGET_OUTPUT_ENV: &str = "WF_INFO_OVERLAY_OUTPUT";

pub(crate) fn run_reward_overlay_from_args(
    args: impl IntoIterator<Item = OsString>,
) -> Option<Result<(), String>> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some(std::ffi::OsStr::new(REWARD_OVERLAY_ARG)) {
        return None;
    }

    Some(run_reward_overlay(args))
}

pub(crate) fn run_test_reward_overlay_from_args(
    args: impl IntoIterator<Item = OsString>,
) -> Option<Result<(), String>> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some(std::ffi::OsStr::new(TEST_REWARD_OVERLAY_ARG)) {
        return None;
    }

    Some(run_test_reward_overlay())
}

pub(crate) fn spawn_reward_overlay(
    output: &ScanOutput,
    output_size: Option<(u32, u32)>,
) -> Result<(), String> {
    if output.kind != ScanKind::Reward || output.items.is_empty() {
        return Ok(());
    }

    let output_size = output_size.unwrap_or((output.source_width, output.source_height));
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(exe);
    command
        .arg(REWARD_OVERLAY_ARG)
        .arg(OUTPUT_SIZE_ARG)
        .arg(format!("{}x{}", output_size.0, output_size.1))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(target_output) = std::env::var_os(TARGET_OUTPUT_ENV) {
        command.arg(TARGET_OUTPUT_ARG).arg(target_output);
    }

    for item in output.items.iter().take(4) {
        command.arg(reward_arg(item));
    }

    command.spawn().map_err(|error| error.to_string())?;

    Ok(())
}

pub(crate) fn spawn_test_reward_overlay() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(exe);
    command
        .arg(TEST_REWARD_OVERLAY_ARG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.spawn().map_err(|error| error.to_string())?;

    Ok(())
}

fn run_test_reward_overlay() -> Result<(), String> {
    let output = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?
        .block_on(display_outputs())?
        .into_iter()
        .max_by_key(|output| {
            (
                output.position.0.saturating_add(output.size.0 as i32),
                output.position.0,
                output.size.0,
                output.size.1,
            )
        })
        .ok_or_else(|| "no display outputs were selected".to_owned())?;

    run_overlay(RewardOverlay {
        output_name: output.name,
        output_size: Some(output.size),
        duration: Some(Duration::from_secs(5)),
        rewards: test_rewards(),
    })
}

fn run_reward_overlay(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let mut output_name = None;
    let mut output_size = None;
    let mut target_output = None;
    let mut rewards = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == OUTPUT_SIZE_ARG {
            let Some(value) = args.next() else {
                return Err("missing reward overlay output size".to_owned());
            };
            output_size = Some(parse_output_size(&value)?);
        } else if arg == TARGET_OUTPUT_ARG {
            let Some(value) = args.next() else {
                return Err("missing reward overlay target output".to_owned());
            };
            target_output = Some(value.to_string_lossy().into_owned());
        } else {
            rewards.push(parse_reward_arg(&arg));
        }
    }

    if rewards.is_empty() {
        return Err("reward overlay needs at least one reward".to_owned());
    }

    if let Some(target_output) = target_output {
        let outputs = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?
            .block_on(display_outputs())?;
        let Some(output) = outputs
            .into_iter()
            .find(|output| output.matches_name(&target_output))
        else {
            return Err(format!(
                "overlay target output {target_output:?} was not found"
            ));
        };

        output_name = output.name;
        output_size = Some(output.size);
    }

    run_overlay(RewardOverlay {
        output_name,
        output_size,
        duration: None,
        rewards,
    })
}

#[cfg(target_os = "linux")]
async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
    overlay_wayland::display_outputs().await
}

#[cfg(not(target_os = "linux"))]
async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
    Err("Wayland reward overlays are not supported on this platform".to_owned())
}

#[cfg(target_os = "linux")]
fn run_overlay(reward_overlay: RewardOverlay) -> Result<(), String> {
    overlay_wayland::run(reward_overlay).map_err(|error| error.to_string())
}

#[cfg(not(target_os = "linux"))]
fn run_overlay(_reward_overlay: RewardOverlay) -> Result<(), String> {
    Err("Wayland reward overlays are not supported on this platform".to_owned())
}

fn reward_arg(item: &WarframeItem) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}",
        item.drop_name,
        item.platinum_rounded(),
        item.ducats
            .map(|ducats| ducats.to_string())
            .unwrap_or_default(),
        item.volume,
        if item.vaulted { "1" } else { "0" }
    )
}

fn parse_reward_arg(arg: &OsString) -> RewardOverlayEntry {
    let value = arg.to_string_lossy();
    let fields = value.split('\t').collect::<Vec<_>>();

    if fields.len() != 5 {
        return RewardOverlayEntry::name_only(value);
    }

    let mut reward = RewardOverlayEntry::name_only(fields[0]);

    if let Ok(platinum) = fields[1].parse() {
        reward = reward.with_platinum(platinum);
    }
    if let Ok(ducats) = fields[2].parse() {
        reward = reward.with_ducats(ducats);
    }
    if let Ok(volume) = fields[3].parse() {
        reward = reward.with_volume(volume);
    }

    reward.with_vaulted(fields[4] == "1")
}

fn parse_output_size(value: &OsString) -> Result<(u32, u32), String> {
    let value = value.to_string_lossy();
    let Some((width, height)) = value.split_once('x') else {
        return Err(format!("invalid reward overlay output size: {value}"));
    };

    let width = width
        .parse()
        .map_err(|error| format!("invalid reward overlay output width: {error}"))?;
    let height = height
        .parse()
        .map_err(|error| format!("invalid reward overlay output height: {error}"))?;

    Ok((width, height))
}

fn test_rewards() -> Vec<RewardOverlayEntry> {
    vec![
        RewardOverlayEntry::name_only("Forma Blueprint")
            .with_platinum(8)
            .with_ducats(0)
            .with_volume(172),
        RewardOverlayEntry::name_only("Braton Prime Receiver")
            .with_platinum(42)
            .with_ducats(45)
            .with_volume(18)
            .with_vaulted(true),
        RewardOverlayEntry::name_only("Paris Prime String")
            .with_platinum(15)
            .with_ducats(25)
            .with_volume(36),
        {
            let mut reward = RewardOverlayEntry::name_only("Akbronco Prime Link")
                .with_platinum(24)
                .with_ducats(45)
                .with_volume(7);
            reward.highlight = RewardHighlight::BestPlatinum;
            reward
        },
    ]
}

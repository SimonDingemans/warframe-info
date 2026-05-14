use std::{
    ffi::OsString,
    process::{Command, Stdio},
};

use wf_info_core::{ScanKind, ScanOutput};
use wf_info_overlay::{RewardOverlay, RewardOverlayEntry};

const REWARD_OVERLAY_ARG: &str = "--wf-info-reward-overlay";
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

pub(crate) fn spawn_reward_overlay(output: &ScanOutput) -> Result<(), String> {
    if output.kind != ScanKind::Reward || output.items.is_empty() {
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(exe);
    command
        .arg(REWARD_OVERLAY_ARG)
        .arg(OUTPUT_SIZE_ARG)
        .arg(format!("{}x{}", output.source_width, output.source_height))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(target_output) = std::env::var_os(TARGET_OUTPUT_ENV) {
        command.arg(TARGET_OUTPUT_ARG).arg(target_output);
    }

    for item in output.items.iter().take(4) {
        command.arg(item);
    }

    command.spawn().map_err(|error| error.to_string())?;

    Ok(())
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
            rewards.push(RewardOverlayEntry::name_only(arg.to_string_lossy()));
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
            .block_on(wf_info_overlay::display_outputs())?;
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

    wf_info_overlay::run(RewardOverlay {
        output_name,
        output_size,
        duration: None,
        rewards,
    })
    .map_err(|error| error.to_string())
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

mod app;
mod hotkeys;
mod market;
mod overlay;
mod scan;

use std::process::ExitCode;

use wf_info_core::SettingsPaths;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    if let Some(result) = market::run_cache_command_from_args(std::env::args_os().skip(1)) {
        return result;
    }

    if let Some(result) = overlay::run_test_reward_overlay_from_args(std::env::args_os().skip(1)) {
        return result;
    }

    if let Some(result) = overlay::run_reward_overlay_from_args(std::env::args_os().skip(1)) {
        return result;
    }

    let settings_path = SettingsPaths::detect().settings_file;

    app::run(settings_path)
}

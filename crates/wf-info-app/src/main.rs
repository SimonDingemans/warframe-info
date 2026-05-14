mod app;
mod hotkeys;
mod scan;

use std::{process::ExitCode};

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
    let settings_path = SettingsPaths::detect().settings_file;

    app::run(settings_path)
}

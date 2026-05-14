use wf_info_core::{scan_image, ScanKind, ScanOutput};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScanReport {
    pub(crate) output: ScanOutput,
    pub(crate) overlay_output_size: Option<(u32, u32)>,
}

pub(crate) fn should_request_screen_capture_permission() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some()
}

pub(crate) async fn request_screen_capture_permission() -> Result<(), String> {
    wf_info_capture::request_screen_capture_permission()
        .await
        .map_err(|error| error.to_string())
}

pub(crate) fn reset_screen_capture_restore_token() -> Result<(), String> {
    let mut errors = Vec::new();

    if let Err(error) = wf_info_capture::reset_screen_capture_restore_token() {
        errors.push(error.to_string());
    }

    if let Err(error) = wf_info_overlay::reset_display_restore_token() {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub(crate) async fn run_scan(kind: ScanKind) -> Result<ScanReport, String> {
    let screenshot = wf_info_capture::capture_screen()
        .await
        .map_err(|error| error.to_string())?;
    let overlay_output_size = screenshot.source.map(|source| source.size);
    let output = scan_image(kind, &screenshot.image).map_err(|error| error.to_string())?;

    Ok(ScanReport {
        output,
        overlay_output_size,
    })
}

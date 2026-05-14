use wf_info_core::{scan_image, ScanKind, ScanOutput};

pub(crate) async fn run_scan(kind: ScanKind) -> Result<ScanOutput, String> {
    let screenshot = wf_info_capture::capture_screen()
        .await
        .map_err(|error| error.to_string())?;

    scan_image(kind, &screenshot.image).map_err(|error| error.to_string())
}

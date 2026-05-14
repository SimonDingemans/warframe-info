use info_core::{scan_image_with_item_database, ScanKind, ScanOutput};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScanReport {
    pub(crate) output: ScanOutput,
    pub(crate) overlay_output_size: Option<(u32, u32)>,
}

pub(crate) fn should_request_screen_capture_permission() -> bool {
    screen_capture().capabilities().permission_request
}

pub(crate) async fn request_screen_capture_permission() -> Result<(), String> {
    screen_capture()
        .request_permission()
        .await
        .map_err(|error| error.to_string())
}

pub(crate) fn reset_screen_capture_restore_token() -> Result<(), String> {
    let mut errors = Vec::new();

    if let Err(error) = screen_capture().reset_permission_state() {
        errors.push(error.to_string());
    }

    if let Err(error) = reset_overlay_display_restore_token() {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(target_os = "linux")]
fn reset_overlay_display_restore_token() -> Result<(), String> {
    overlay_wayland::reset_display_restore_token()
}

#[cfg(not(target_os = "linux"))]
fn reset_overlay_display_restore_token() -> Result<(), String> {
    Ok(())
}

pub(crate) async fn run_scan(kind: ScanKind) -> Result<ScanReport, String> {
    let screenshot = screen_capture()
        .capture_screen()
        .await
        .map_err(|error| error.to_string())?;
    let overlay_output_size = screenshot.source.map(|source| source.size);
    let mut market = crate::market::MarketData::load().await?;
    let output = scan_image_with_item_database(kind, &screenshot.image, &market.database)
        .map_err(|error| error.to_string())?;
    let output = market.enrich_scan_output(output).await;

    Ok(ScanReport {
        output,
        overlay_output_size,
    })
}

fn screen_capture() -> Box<dyn capture::ScreenCapture> {
    #[cfg(target_os = "linux")]
    {
        Box::new(capture_wayland::WaylandCapture::new())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Box::new(capture::UnsupportedCapture)
    }
}

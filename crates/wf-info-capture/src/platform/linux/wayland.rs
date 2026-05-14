use std::{ffi::OsString, path::PathBuf};

use ashpd::desktop::screenshot::Screenshot as PortalScreenshot;

use crate::{CaptureError, CaptureFuture, CaptureResult, ScreenCapture, Screenshot};

#[derive(Debug, Clone, Default)]
pub struct LinuxWaylandCapture;

impl LinuxWaylandCapture {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenCapture for LinuxWaylandCapture {
    fn capture_screen(&self) -> CaptureFuture<'_> {
        Box::pin(async { capture_screen_with_portal().await })
    }
}

async fn capture_screen_with_portal() -> CaptureResult<Screenshot> {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(CaptureError::NotWaylandSession);
    }

    let response = PortalScreenshot::request()
        .interactive(false)
        .modal(false)
        .send()
        .await?
        .response()?;
    let path = file_uri_to_path(response.uri().as_str())?;
    let image = image::open(&path).map_err(|source| CaptureError::OpenImage { path, source })?;

    Ok(Screenshot { image })
}

fn file_uri_to_path(uri: &str) -> CaptureResult<PathBuf> {
    let path =
        uri.strip_prefix("file://")
            .ok_or_else(|| CaptureError::UnsupportedScreenshotUri {
                uri: uri.to_string(),
            })?;

    if path.is_empty() || path.starts_with('/') {
        return percent_decode_path(path, uri);
    }

    let Some(local_path) = path.strip_prefix("localhost/") else {
        return Err(CaptureError::UnsupportedScreenshotUri {
            uri: uri.to_string(),
        });
    };

    percent_decode_path(&format!("/{local_path}"), uri)
}

fn percent_decode_path(path: &str, uri: &str) -> CaptureResult<PathBuf> {
    let mut bytes = Vec::with_capacity(path.len());
    let mut chars = path.as_bytes().iter().copied();

    while let Some(byte) = chars.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }

        let Some(high) = chars.next() else {
            return Err(CaptureError::InvalidScreenshotUri {
                uri: uri.to_string(),
            });
        };
        let Some(low) = chars.next() else {
            return Err(CaptureError::InvalidScreenshotUri {
                uri: uri.to_string(),
            });
        };

        let Some(decoded) = decode_hex_pair(high, low) else {
            return Err(CaptureError::InvalidScreenshotUri {
                uri: uri.to_string(),
            });
        };
        bytes.push(decoded);
    }

    let path = String::from_utf8(bytes).map_err(|_| CaptureError::InvalidScreenshotUri {
        uri: uri.to_string(),
    })?;
    Ok(PathBuf::from(OsString::from(path)))
}

fn decode_hex_pair(high: u8, low: u8) -> Option<u8> {
    Some(hex_value(high)? * 16 + hex_value(low)?)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_to_path_decodes_absolute_paths() {
        assert_eq!(
            file_uri_to_path("file:///tmp/Warframe%20Shot.png").unwrap(),
            PathBuf::from("/tmp/Warframe Shot.png")
        );
    }

    #[test]
    fn file_uri_to_path_rejects_non_file_uris() {
        assert!(matches!(
            file_uri_to_path("https://example.com/screenshot.png"),
            Err(CaptureError::UnsupportedScreenshotUri { .. })
        ));
    }
}

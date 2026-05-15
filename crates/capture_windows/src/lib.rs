#[cfg(not(target_os = "windows"))]
use capture::CaptureError;
use capture::{CaptureFuture, CaptureResult, ScreenCapture, Screenshot};

#[derive(Debug, Clone, Default)]
pub struct WindowsCapture;

impl WindowsCapture {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenCapture for WindowsCapture {
    fn capture_screen(&self) -> CaptureFuture<'_> {
        Box::pin(async { capture_screen() })
    }
}

#[cfg(target_os = "windows")]
fn capture_screen() -> CaptureResult<Screenshot> {
    windows::capture_screen()
}

#[cfg(not(target_os = "windows"))]
fn capture_screen() -> CaptureResult<Screenshot> {
    Err(CaptureError::UnsupportedBackend)
}

#[cfg(target_os = "windows")]
mod windows {
    use std::{ffi::c_void, mem, ptr};

    use capture::{CaptureError, CaptureResult, ScreenCaptureSource, Screenshot};
    use image::{DynamicImage, RgbaImage};
    use windows_sys::Win32::{
        Foundation::{GetLastError, HWND},
        Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
            GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
            HBITMAP, HDC, HGDIOBJ, SRCCOPY,
        },
        UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        },
    };

    pub(super) fn capture_screen() -> CaptureResult<Screenshot> {
        let area = virtual_screen_area()?;
        let frame = capture_virtual_screen(area)?;
        let image = bgra_to_rgba_image(&frame, area.width, area.height)?;

        Ok(Screenshot {
            image: DynamicImage::ImageRgba8(image),
            source: Some(ScreenCaptureSource {
                size: (area.width, area.height),
            }),
        })
    }

    #[derive(Debug, Clone, Copy)]
    struct VirtualScreenArea {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    }

    fn virtual_screen_area() -> CaptureResult<VirtualScreenArea> {
        let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if width <= 0 || height <= 0 {
            return Err(CaptureError::SourceUnavailable {
                message: "Windows did not report a virtual screen to capture".to_owned(),
            });
        }

        Ok(VirtualScreenArea {
            x: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            y: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
            width: width as u32,
            height: height as u32,
        })
    }

    fn capture_virtual_screen(area: VirtualScreenArea) -> CaptureResult<Vec<u8>> {
        let screen_dc = WindowDc::desktop()?;
        let memory_dc = MemoryDc::compatible(screen_dc.as_raw())?;
        let bitmap = Bitmap::compatible(screen_dc.as_raw(), area.width, area.height)?;

        {
            let _selection = SelectedObject::select(memory_dc.as_raw(), bitmap.as_object())?;
            let copied = unsafe {
                BitBlt(
                    memory_dc.as_raw(),
                    0,
                    0,
                    area.width as i32,
                    area.height as i32,
                    screen_dc.as_raw(),
                    area.x,
                    area.y,
                    SRCCOPY | CAPTUREBLT,
                )
            };
            if copied == 0 {
                return Err(windows_capture_error("BitBlt"));
            }
        }

        read_bitmap(memory_dc.as_raw(), bitmap.as_raw(), area.width, area.height)
    }

    fn read_bitmap(hdc: HDC, bitmap: HBITMAP, width: u32, height: u32) -> CaptureResult<Vec<u8>> {
        let byte_len = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(CaptureError::InvalidFrame)? as usize;
        let mut pixels = vec![0; byte_len];
        let mut info = bitmap_info(width, height)?;

        let scanlines = unsafe {
            GetDIBits(
                hdc,
                bitmap,
                0,
                height,
                pixels.as_mut_ptr().cast::<c_void>(),
                &mut info,
                DIB_RGB_COLORS,
            )
        };
        if scanlines == 0 || scanlines as u32 != height {
            return Err(windows_capture_error("GetDIBits"));
        }

        Ok(pixels)
    }

    fn bitmap_info(width: u32, height: u32) -> CaptureResult<BITMAPINFO> {
        let width = i32::try_from(width).map_err(|_| CaptureError::InvalidFrame)?;
        let height = i32::try_from(height).map_err(|_| CaptureError::InvalidFrame)?;
        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize =
            mem::size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width;
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;

        Ok(info)
    }

    fn bgra_to_rgba_image(frame: &[u8], width: u32, height: u32) -> CaptureResult<RgbaImage> {
        let expected_len = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(CaptureError::InvalidFrame)? as usize;
        if frame.len() != expected_len {
            return Err(CaptureError::InvalidFrame);
        }

        let mut rgba = Vec::with_capacity(expected_len);
        for pixel in frame.chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
        }

        RgbaImage::from_raw(width, height, rgba).ok_or(CaptureError::InvalidFrame)
    }

    #[derive(Debug)]
    struct WindowDc {
        hwnd: HWND,
        hdc: HDC,
    }

    impl WindowDc {
        fn desktop() -> CaptureResult<Self> {
            let hwnd = ptr::null_mut();
            let hdc = unsafe { GetDC(hwnd) };
            if hdc.is_null() {
                return Err(windows_capture_error("GetDC"));
            }

            Ok(Self { hwnd, hdc })
        }

        fn as_raw(&self) -> HDC {
            self.hdc
        }
    }

    impl Drop for WindowDc {
        fn drop(&mut self) {
            unsafe {
                ReleaseDC(self.hwnd, self.hdc);
            }
        }
    }

    #[derive(Debug)]
    struct MemoryDc(HDC);

    impl MemoryDc {
        fn compatible(hdc: HDC) -> CaptureResult<Self> {
            let memory_dc = unsafe { CreateCompatibleDC(hdc) };
            if memory_dc.is_null() {
                return Err(windows_capture_error("CreateCompatibleDC"));
            }

            Ok(Self(memory_dc))
        }

        fn as_raw(&self) -> HDC {
            self.0
        }
    }

    impl Drop for MemoryDc {
        fn drop(&mut self) {
            unsafe {
                DeleteDC(self.0);
            }
        }
    }

    #[derive(Debug)]
    struct Bitmap(HBITMAP);

    impl Bitmap {
        fn compatible(hdc: HDC, width: u32, height: u32) -> CaptureResult<Self> {
            let bitmap = unsafe { CreateCompatibleBitmap(hdc, width as i32, height as i32) };
            if bitmap.is_null() {
                return Err(windows_capture_error("CreateCompatibleBitmap"));
            }

            Ok(Self(bitmap))
        }

        fn as_raw(&self) -> HBITMAP {
            self.0
        }

        fn as_object(&self) -> HGDIOBJ {
            self.0
        }
    }

    impl Drop for Bitmap {
        fn drop(&mut self) {
            unsafe {
                DeleteObject(self.0);
            }
        }
    }

    #[derive(Debug)]
    struct SelectedObject {
        hdc: HDC,
        previous: HGDIOBJ,
    }

    impl SelectedObject {
        fn select(hdc: HDC, object: HGDIOBJ) -> CaptureResult<Self> {
            let previous = unsafe { SelectObject(hdc, object) };
            if previous.is_null() {
                return Err(windows_capture_error("SelectObject"));
            }

            Ok(Self { hdc, previous })
        }
    }

    impl Drop for SelectedObject {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.hdc, self.previous);
            }
        }
    }

    fn windows_capture_error(operation: &'static str) -> CaptureError {
        let code = unsafe { GetLastError() };
        CaptureError::FrameCaptureFailed {
            message: format!(
                "Windows screen capture failed during {operation} (GetLastError={code})"
            ),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn converts_bgra_frame_to_rgba_image() {
            let image = bgra_to_rgba_image(&[10, 20, 30, 0, 40, 50, 60, 128], 2, 1).unwrap();

            assert_eq!(image.as_raw(), &[30, 20, 10, 255, 60, 50, 40, 255]);
        }
    }
}

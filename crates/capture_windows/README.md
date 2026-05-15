# capture_windows

`capture_windows` implements the shared `capture::ScreenCapture` trait for
Windows.

The backend captures the Windows virtual desktop with GDI and returns a
backend-neutral `capture::Screenshot`.

## Usage

```rust
use capture::ScreenCapture;
use capture_windows::WindowsCapture;

let backend = WindowsCapture::new();
let screenshot = backend.capture_screen().await?;
```

Run the example on Windows:

```sh
cargo run -p capture_windows --example capture
```

The example writes a PNG to `crates/capture_windows/examples/tmp`.

# wf-info-capture

`wf-info-capture` captures the current Warframe window as an `image::DynamicImage`.

The crate currently targets Linux Wayland. It uses the desktop screencast portal to ask for a window, reads one PipeWire frame, and returns it as a `Screenshot`. A restore token is cached so later captures can reuse the portal selection when the desktop allows it.

## What It Can Do

- Capture a single screen/window frame asynchronously.
- Hide the cursor from the captured image.
- Return structured errors for unsupported platforms, missing Wayland sessions, portal failures, and PipeWire frame issues.
- Provide a `ScreenCapture` trait so other capture backends can be plugged in later.

## Usage

```rust
let screenshot = wf_info_capture::capture_screen().await?;
screenshot.image.save("warframe.png")?;
```

Run the example:

```sh
cargo run -p wf-info-capture --example capture
```

The example writes a PNG to `crates/wf-info-capture/examples/tmp`.

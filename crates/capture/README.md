# capture

`capture` captures the selected screen as an `image::DynamicImage`.

The crate currently targets Linux Wayland. It uses the desktop screencast portal to ask for a screen, reads one PipeWire frame, and returns it as a `Screenshot`. A restore token is cached so later captures can reuse the portal selection when the desktop allows it.

## What It Can Do

- Capture a single screen frame asynchronously.
- Hide the cursor from the captured image.
- Return structured errors for unsupported platforms, missing Wayland sessions, portal failures, and PipeWire frame issues.
- Provide a `ScreenCapture` trait so other capture backends can be plugged in later.

## Usage

```rust
let screenshot = capture::capture_screen().await?;
screenshot.image.save("warframe.png")?;
```

Run the example:

```sh
cargo run -p capture --example capture
```

The example writes a PNG to `crates/capture/examples/tmp`.

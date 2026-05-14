# capture

`capture` defines the shared screen-capture abstraction and data types.

Backend implementations live in separate crates, such as `capture_wayland`.

## What It Can Do

- Define the `ScreenCapture` trait for backend implementations.
- Carry captured images as `Screenshot` values.
- Preserve optional source monitor metadata.
- Describe backend capabilities such as permission request/reset support.
- Return backend-neutral capture errors.
- Provide `UnsupportedCapture` for platforms without a capture backend yet.

## Usage

```rust
let screenshot = backend.capture_screen().await?;
screenshot.image.save("warframe.png")?;
```

Use a backend crate or `platform_capabilities::screen_capture::backend()` to
construct `backend`.

# capture_wayland

`capture_wayland` implements the `capture::ScreenCapture` trait for Linux Wayland sessions.

It uses the desktop screencast portal to ask for a screen, reads one PipeWire frame, and returns it as a `capture::Screenshot`. A restore token is cached so later captures can reuse the portal selection when the desktop allows it.

## What It Can Do

- Request monitor capture through the XDG desktop screencast portal.
- Reuse the portal restore token for later captures.
- Reset the stored restore token.
- Convert common PipeWire RGB/BGR frame formats into `image::DynamicImage`.

The restore token is stored at
`${XDG_CACHE_HOME:-~/.cache}/warframe-info/wayland-monitor-screencast-token`.

## Usage

```rust
use capture::ScreenCapture;
use capture_wayland::WaylandCapture;

let backend = WaylandCapture::new();
let screenshot = backend.capture_screen().await?;
```

Run the example:

```sh
cargo run -p capture_wayland --example capture
```

The example writes a PNG to `crates/capture_wayland/examples/tmp`.

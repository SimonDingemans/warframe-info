# capture_wayland

`capture_wayland` implements the `capture::ScreenCapture` trait for Linux Wayland sessions.

It uses the desktop screencast portal to ask for a screen, reads one PipeWire frame, and returns it as a `capture::Screenshot`. A restore token is cached so later captures can reuse the portal selection when the desktop allows it.

Run the example:

```sh
cargo run -p capture_wayland --example capture
```

The example writes a PNG to `crates/capture_wayland/examples/tmp`.

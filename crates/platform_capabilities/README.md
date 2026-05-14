# platform_capabilities

`platform_capabilities` selects the concrete backend implementations available
on the current platform.

The app uses this crate instead of depending directly on platform-specific
capture, hotkey, and overlay crates.

## What It Can Do

- Select the app hotkey backend.
- Expose system shortcut configuration when supported.
- Select the screen capture backend.
- List reward overlay display outputs.
- Run or reset the reward overlay backend.

## Current Behavior

- Linux Wayland: uses `capture_wayland`, `hotkeys_wayland`, and
  `overlay_wayland`.
- Linux non-Wayland: returns unsupported capture and overlay behavior.
- Other platforms: uses the native `hotkeys` backend, with capture and overlay
  still unsupported.

## Usage

```rust
let capture = platform_capabilities::screen_capture::backend();
let hotkeys = platform_capabilities::global_shortcuts::backend();
```

For overlays:

```rust
platform_capabilities::reward_overlay::run(overlay)?;
```

# hotkeys

`hotkeys` defines the app-facing global shortcut abstraction.

It provides native global hotkey registration where available, an integration
hook for desktop-managed shortcut systems, and an `iced` subscription that turns
shortcut activations into `info_core::HotkeyEvent` values.

## What It Can Do

- Register reward and inventory scan hotkeys with `global-hotkey`.
- Parse hotkey strings from `AppSettings`.
- Expose integrated shortcut backends through `ShortcutIntegration`.
- Stream shortcut events into the UI.
- Report unsupported shortcut configuration through `unsupported`.

## Usage

```rust
use hotkeys::{HotkeyBackend, HotkeyBindings};
use info_core::AppSettings;

let settings = AppSettings::default();
let (bindings, status) = HotkeyBindings::new(&settings, HotkeyBackend::Native);
let subscription = bindings.subscription(&settings);
```

Use `platform_capabilities::global_shortcuts::backend()` when the app should
choose the backend for the current platform.

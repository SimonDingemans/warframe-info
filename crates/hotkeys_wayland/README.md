# hotkeys_wayland

`hotkeys_wayland` integrates Warframe Info hotkeys with the XDG desktop portal
global shortcuts API.

It implements `hotkeys::ShortcutIntegration` so the app can configure and watch
desktop-managed shortcuts without depending directly on `ashpd`.

## What It Can Do

- Register reward and inventory scan actions with the desktop portal.
- Request preferred triggers from the current app settings.
- Open desktop shortcut configuration on portal version 2.
- Try to open KDE shortcut settings when only portal version 1 is available.
- Watch portal activation events and emit `HotkeyEvent::Triggered` values.

## Usage

```rust
let backend = hotkeys::HotkeyBackend::Integrated(
    hotkeys_wayland::shortcut_integration(),
);
```

Most callers should use `platform_capabilities::global_shortcuts::backend()` so
Wayland detection stays in one place.

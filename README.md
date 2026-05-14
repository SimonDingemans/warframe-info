# warframe-info

`warframe-info` is a cross-platform Warframe companion app in early development.
It captures the game screen, crops known Warframe UI regions, runs OCR over the
cropped image, matches recognized text against Warframe Market items, and shows
reward information in an overlay where the platform supports it.

The current implementation is most complete on Linux Wayland. Other platforms
keep the same crate boundaries, but several backends are still placeholders.

## Workspace

- `app`: the desktop UI and scan orchestrator.
- `info_core`: shared settings, item matching, and scan pipeline glue.
- `capture`: backend-neutral screen capture traits and data types.
- `capture_wayland`: Wayland screencast portal and PipeWire capture backend.
- `crop`: Warframe reward and inventory crop logic.
- `ocr`: ONNX OCR engine, text normalization, and screen layouts.
- `hotkeys`: backend-neutral global shortcut wiring plus the native backend.
- `hotkeys_wayland`: XDG desktop portal global shortcut integration.
- `overlay`: shared reward overlay data types and display backend traits.
- `overlay_wayland`: Wayland layer-shell reward overlay.
- `platform_capabilities`: selects the available backend implementations for the
  current platform.

## Running the App

```sh
cargo run -p app
```

The app opens an `iced` UI with scan and settings tabs. From the UI you can:

- Run reward or inventory scans.
- Save reward and inventory hotkey settings.
- Configure Wayland desktop shortcuts when the portal supports it.
- Reset the Wayland screen capture restore token.
- Clear the Warframe Market cache.
- Spawn a test reward overlay.

Reward scans also try to show the Wayland reward overlay automatically when
rewards are found.

## Settings and Caches

Settings are saved as TOML in the platform config directory:

- Linux: `${XDG_CONFIG_HOME:-~/.config}/warframe-info/settings.toml`
- Windows: `%APPDATA%/warframe-info/settings.toml`

Default settings:

```toml
[hotkeys]
reward_scan = "Ctrl+Shift+R"
inventory_scan = "Ctrl+Shift+I"
```

Warframe Market item and price data is cached in
`${XDG_CACHE_HOME:-~/.cache}/wf-info/`:

- `wf_market_cache.json`: item index cache.
- `wf_market_price_cache.json`: top sell price cache.

Both caches are treated as fresh for 1 hour. Clear them from the app with
`Clear Market Cache`, or from the command line:

```sh
cargo run -p app -- cache clear
```

Wayland screencast restore tokens are stored separately in
`${XDG_CACHE_HOME:-~/.cache}/warframe-info/wayland-monitor-screencast-token`.

## Platform Notes

On Wayland, screen capture and display selection use the XDG desktop screencast
portal. Global shortcuts use the XDG desktop portal global shortcuts API. The
first capture or shortcut configuration may open a desktop permission dialog.

Portal version 2 can open the desktop shortcut configuration flow directly.
Portal version 1 can register and list shortcuts, but if the desktop reports
them as `unassigned`, assign the listed actions manually in the desktop shortcut
settings. On KDE Plasma, the app tries to open the Shortcuts settings page.

On non-Wayland Linux sessions, capture and overlay support currently report that
Wayland is required. On non-Linux targets, the native hotkey backend is used, but
screen capture and overlays are not implemented yet.

## Development

Run the full workspace tests:

```sh
cargo test --workspace
```

Run examples for the lower-level crates:

```sh
cargo run -p crop --example crop_inventory
cargo run -p crop --example crop_reward_screen
cargo run -p ocr --example ocr_inventory
cargo run -p ocr --example ocr_reward_screen
cargo run -p capture_wayland --example capture
cargo run -p overlay_wayland --example reward_overlay
```

The dependency direction is intentionally layered: `app` composes platform
capabilities, `info_core` owns shared domain logic, and platform-specific crates
hide desktop APIs behind small traits.

# warframe-info

`warframe-info` is a cross-platform Warframe companion app in early development.
It captures the game screen, crops known Warframe UI regions, runs OCR over the
cropped image, matches recognized text against Warframe Market items, and shows
reward information in an overlay where the platform supports it.

The current implementation is most complete on Linux Wayland. Other platforms
keep the same crate boundaries, but several backends are still placeholders.

## Related Projects

`warframe-info` is based on ideas and workflows already explored by existing
Warframe companion tools:

- [WFCD/WFinfo](https://github.com/WFCD/WFinfo): a fissure companion app for
  Warframe that screenshots the game, crops reward text, runs OCR, looks up
  Platinum and Ducat values, and shows the result in an overlay or separate
  window.
- [knoellle/wfinfo-ng](https://github.com/knoellle/wfinfo-ng): a Linux-compatible
  take on WFinfo that detects relic reward screens, captures the game, identifies
  reward items, and displays Platinum values on X11 and Wayland.

This project follows the same broad problem shape, but is being built as a new
Rust workspace with small platform backends, an ONNX OCR pipeline, and Wayland
portal/layer-shell support as the first-class path.

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

## Tools

The [tools](tools) directory contains development-only helper projects. These
are not part of the runtime app.

`tools/ocr-models` is a Pixi workspace for regenerating the PaddleOCR ONNX model
files used by the `ocr` crate. It pins a Linux Python environment with
`paddleocr`, `paddlepaddle`, `paddle2onnx`, and `huggingface-hub`.

```sh
cd tools/ocr-models
pixi run download-models
pixi run export-onnx
```

The tasks download PaddleOCR detector and recognizer models from Hugging Face,
then export them to ONNX. The checked-in OCR crate currently loads
`crates/ocr/assets/ocr/det_model.onnx` and
`crates/ocr/assets/ocr/rec_model.onnx`; make sure regenerated models are placed
there before testing or committing them.

The dependency direction is intentionally layered: `app` composes platform
capabilities, `info_core` owns shared domain logic, and platform-specific crates
hide desktop APIs behind small traits.

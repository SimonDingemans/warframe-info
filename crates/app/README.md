# app

`app` is the desktop process for Warframe Info.

It owns the `iced` UI, loads settings, watches global hotkeys, runs the capture,
crop, OCR, and item matching pipeline, enriches results with Warframe Market
data, and asks the platform overlay backend to display reward results.

## What It Can Do

- Open the scan and settings UI.
- Run reward and inventory scans manually.
- Start scans from configured global hotkeys.
- Save hotkey settings to the platform config directory.
- Configure Wayland portal shortcuts when supported.
- Request or reset Wayland screen capture permission state.
- Cache Warframe Market item and price data.
- Spawn a reward overlay or a test overlay through `platform_capabilities`.

## Usage

```sh
cargo run -p app
```

Clear Warframe Market caches:

```sh
cargo run -p app -- cache clear
```

The app stores settings in:

- Linux: `${XDG_CONFIG_HOME:-~/.config}/warframe-info/settings.toml`
- Windows: `%APPDATA%/warframe-info/settings.toml`

Warframe Market caches live in `${XDG_CACHE_HOME:-~/.cache}/wf-info/` and are
treated as fresh for 1 hour.

## Platform Notes

On Linux Wayland, capture, shortcuts, and overlays are routed through portal and
layer-shell backends. On other platforms, the app still starts, but capture and
overlay support depend on future backend crates.

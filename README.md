# warframe-info

`warframe-info` is being built as a cross-platform Warframe companion app.

The current workspace contains focused library crates for capture, cropping, and OCR. The next step is to introduce application-level crates that compose those libraries into a normal desktop app and a separate overlay process.

## Planned Crates

The names below are placeholders and can change once the shape of the app settles.

### `app`

The normal desktop application process.

Responsibilities:

- Own global hotkey registration.
- Run capture, crop, OCR, and item processing pipelines when a hotkey is pressed.
- Hold user configuration and app lifecycle state.
- Start, stop, or reconnect to the overlay process.
- Send overlay updates through a small IPC boundary.

This crate should avoid platform-specific overlay rendering code. It can depend on platform-specific hotkey and capture backends, but its job is orchestration rather than presentation.

Initial commands:

```sh
cargo run -p app
cargo run -p app -- settings show
cargo run -p app -- settings set-reward-hotkey "Ctrl+Shift+R"
cargo run -p app -- settings set-inventory-hotkey "Ctrl+Shift+I"
```

Scans use the `wf-market` crate for live warframe.market data. The item index is
cached in `${XDG_CACHE_HOME:-~/.cache}/wf-info/wf_market_cache.json`, and top
sell prices are cached per item in `wf_market_price_cache.json`. Both caches are
treated as fresh for 1 hour. Clear them from the app with `Clear Market Cache`,
or from the command line:

```sh
cargo run -p app -- cache clear
```

Running `app` opens the initial `iced` settings UI. The UI can save hotkey settings and manually run reward or inventory scans through the current capture, crop, and OCR pipeline. The command-line settings commands are kept for scripting and quick checks.

Settings are saved as TOML in the platform config directory by default:

```toml
[hotkeys]
reward_scan = "Ctrl+Shift+R"
inventory_scan = "Ctrl+Shift+I"
```

Global hotkeys are hidden behind a small app-side backend abstraction. The UI subscribes to backend watcher streams instead of relying on button-driven polling:

- Wayland: uses the XDG desktop portal global shortcuts API through `ashpd`.
- Windows and other non-Wayland sessions: use the native `global-hotkey` backend for now.

`ashpd` talks to desktop portals, so it is the right direction for sandbox-friendly Linux desktops and Wayland. It is not expected to provide a Windows backend; Windows support should stay behind the same abstraction and use a Windows-capable implementation.

On Wayland, the app must register shortcuts with the desktop environment before it can receive activations. The UI exposes a `Configure Hotkeys` action that saves the current TOML settings and binds the shortcut IDs through the GlobalShortcuts portal. Portal version 2 can also open the desktop shortcut configuration flow directly. Portal version 1 can register/list shortcuts, but if the desktop reports them as `unassigned`, assign the listed actions manually in the desktop shortcut settings. On KDE Plasma, the app tries to open the Shortcuts settings page automatically.

### `overlay`

The overlay process.

Responsibilities:

- Render the always-on-top overlay UI.
- Receive display commands and data from `app`.
- Handle overlay-specific input behavior, such as click-through or focused interaction modes.
- Hide platform differences behind overlay launch/runtime modules.

The overlay implementation is expected to be platform-specific at the windowing layer:

- Wayland: use `iced` with `iced_layershell` so the overlay can be a real layer-shell surface.
- Windows: use `iced` normal windows with transparent, borderless, always-on-top settings, plus Win32 window styles where needed.
- Other platforms: add backends as the supported behavior becomes clear.

### `info_core`

Shared logic used by both processes.

Responsibilities:

- Define shared domain types, pipeline outputs, and overlay messages.
- Host logic that does not need to know whether it is running in the app or overlay process.
- Provide common configuration models and serialization types.
- Keep IPC payloads stable and explicit.

This crate should not own desktop windows, hotkeys, or OS capture handles. Those remain in app or platform crates so `info_core` stays portable and easy to test.

## Existing Library Crates

These crates remain the low-level building blocks:

- `capture`: captures the Warframe window or screen.
- `crop`: extracts Warframe UI regions from screenshots.
- `ocr`: reads item text from cropped UI images.

The intended dependency direction is:

```text
app
  -> info_core
  -> capture
  -> crop
  -> ocr

overlay
  -> info_core
```

In this sketch, the arrows from `app` are parallel dependencies. The exact graph may change. For example, `info_core` may depend on crop/OCR if it owns full pipeline coordination, while capture may stay app-owned because it touches desktop sessions and OS handles.

## Process Model

```text
Hotkey
  -> app
  -> capture screenshot
  -> crop relevant UI region
  -> run OCR/item pipeline
  -> send result over IPC
  -> overlay
  -> render result
```

Keeping the app and overlay in separate processes gives each side a clearer job:

- The app can keep running even if the overlay backend has to restart.
- The overlay can use platform-specific windowing APIs without leaking those choices into the pipeline code.
- Shared data contracts can be tested without requiring a desktop session.

## Open Decisions

- Final crate names.
- IPC transport between app and overlay.
- Whether `info_core` owns full pipeline execution or only shared types and coordination helpers.
- How much overlay state should live in `info_core` versus `overlay`.
- Packaging strategy for running both processes together on each platform.

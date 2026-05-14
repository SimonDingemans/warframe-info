# info_core

`info_core` contains shared domain logic for Warframe Info.

It stays independent of desktop windows, OS capture handles, and overlay
rendering. The app and backend crates use it for settings, scan types, item
matching, and the image scan pipeline.

## What It Can Do

- Load and save TOML app settings.
- Define scan kinds and hotkey events.
- Crop a screenshot for reward or inventory scans.
- Run OCR over the cropped image.
- Match recognized item text against a fuzzy item database.
- Represent Warframe Market item metadata used by the app and overlay.

## Usage

```rust
use info_core::{scan_image_with_item_database, ItemDatabase, ScanKind};

let database = ItemDatabase::new(items);
let output = scan_image_with_item_database(
    ScanKind::Reward,
    &screenshot,
    &database,
    &mut ocr,
)?;
```

Settings are saved by `AppSettings`:

```rust
use info_core::{AppSettings, SettingsPaths};

let path = SettingsPaths::detect().settings_file;
let settings = AppSettings::load_or_create(path)?;
```

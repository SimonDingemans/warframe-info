# ocr

`ocr` reads item names from cropped Warframe UI images.

It wraps ONNX OCR models with `ort`, detects text bounds, recognizes text, normalizes common Warframe item text, and groups text into items using screen-specific layouts.

## What It Can Do

- Load PaddleOCR detector and recognizer models from `assets/ocr`.
- Detect text boxes and recognize their contents.
- Filter low-confidence OCR results.
- Normalize item text with `WarframeTextNormalizer`.
- Group text into inventory grid items or reward screen items.

## Usage

```rust
use ocr::{
    layouts::InventoryGridLayout,
    load_ocr_engine,
    pipeline::ItemPipeline,
    text::WarframeTextNormalizer,
};

let image = image::open("inventory_cropped.png")?;
let mut ocr = load_ocr_engine()?;
let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
let output = pipeline.run(&mut ocr, &image, &InventoryGridLayout::new(6))?;

for item in output.items {
    println!("{item}");
}
```

For reward screens, use `layouts::RewardScreenLayout::default()`.

Run the examples:

```sh
cargo run -p ocr --example ocr_inventory
cargo run -p ocr --example ocr_reward_screen
```

The examples expect cropped PNG fixtures in `examples/fixtures`.

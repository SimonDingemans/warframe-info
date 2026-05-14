# crop

`crop` extracts Warframe UI regions from screenshots.

It converts resolution-independent ratio rectangles into pixel crops, then returns the cropped image together with metadata about the source size, crop rectangle, and crop kind.

## What It Can Do

- Crop the inventory item grid from a full screenshot.
- Crop the reward card strip from a reward screen.
- Define custom crop rectangles with `RatioRect`.
- Use the `ScreenCrop` trait for shared crop behavior.

## Usage

```rust
use crop::{InventoryCrop, ScreenCrop};

let image = image::open("warframe_inventory.png")?;
let cropped = InventoryCrop::default().crop_image(&image)?;
cropped.image.save("inventory_cropped.png")?;
```

For reward screens, use `RewardScreenCrop::default()` instead.

Run the examples:

```sh
cargo run -p crop --example crop_inventory
cargo run -p crop --example crop_reward_screen
```

The examples read PNG fixtures from `examples/fixtures` and write cropped images to `examples/tmp`.

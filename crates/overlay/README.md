# overlay

`overlay` defines shared reward overlay data and display backend traits.

It does not open windows by itself. Platform-specific crates, such as
`overlay_wayland`, consume these types to render the actual overlay.

## What It Can Do

- Represent reward rows with platinum, ducat, volume, vaulted, ownership, and
  highlight metadata.
- Carry overlay placement hints such as target output and output size.
- Expose display output discovery through `DisplayBackend`.
- Bundle the platinum and ducat icon assets used by overlay renderers.

## Usage

```rust
use overlay::RewardOverlay;
use ui_core::RewardCardEntry;

let overlay = RewardOverlay {
    output_name: None,
    output_size: Some((1920, 1080)),
    duration: None,
    rewards: vec![
        RewardCardEntry::name_only("Forma Blueprint").with_platinum(8),
    ],
};
```

Pass `RewardOverlay` to a platform backend, or call
`platform_capabilities::reward_overlay::run(overlay)` from the app.

# overlay_wayland

`overlay_wayland` renders the reward overlay on Linux Wayland.

It uses `iced_layershell` to create a layer-shell surface and the screencast
portal plus Wayland output metadata to choose the target monitor.

## What It Can Do

- Render up to four reward cards in a transparent layer-shell overlay.
- Position the overlay near the in-game reward card area.
- Auto-close the overlay after a short duration.
- Discover display outputs and match portal streams to Wayland outputs.
- Reset the stored monitor selection restore token.

## Usage

```rust
let overlay = overlay::RewardOverlay {
    output_name: None,
    output_size: Some((1920, 1080)),
    duration: None,
    rewards,
};

overlay_wayland::run(overlay)?;
```

Run the example:

```sh
cargo run -p overlay_wayland --example reward_overlay
```

The monitor restore token is stored at
`${XDG_CACHE_HOME:-~/.cache}/warframe-info/wayland-monitor-screencast-token`.

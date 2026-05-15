use std::time::Duration;

use overlay::{DisplayOutput, RewardOverlay};
use ui_core::{RewardCardEntry, RewardHighlight};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = select_rightmost_output(
        overlay_wayland::display_outputs()
            .await
            .map_err(std::io::Error::other)?,
    )
    .ok_or_else(|| std::io::Error::other("no display outputs were selected"))?;

    eprintln!("{:?}", output);

    overlay_wayland::run(RewardOverlay {
        output_name: output.name,
        output_size: Some(output.size),
        duration: Some(Duration::from_secs(5)),
        rewards: vec![
            RewardCardEntry::name_only("Forma Blueprint")
                .with_platinum(8)
                .with_ducats(0),
            RewardCardEntry::name_only("Braton Prime Receiver")
                .with_platinum(42)
                .with_ducats(45)
                .with_vaulted(true),
            RewardCardEntry::name_only("Paris Prime String")
                .with_platinum(15)
                .with_ducats(25),
            {
                let mut reward = RewardCardEntry::name_only("Akbronco Prime Link")
                    .with_platinum(24)
                    .with_ducats(45);
                reward.highlight = RewardHighlight::BestPlatinum;
                reward
            },
        ],
    })?;

    Ok(())
}

fn select_rightmost_output(outputs: Vec<DisplayOutput>) -> Option<DisplayOutput> {
    outputs.into_iter().max_by_key(|output| {
        (
            output.position.0.saturating_add(output.size.0 as i32),
            output.position.0,
            output.size.0,
            output.size.1,
        )
    })
}

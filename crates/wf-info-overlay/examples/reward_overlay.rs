use std::time::Duration;

use wf_info_overlay::{DisplayOutput, RewardHighlight, RewardOverlay, RewardOverlayEntry};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = select_rightmost_output(
        wf_info_overlay::display_outputs()
            .await
            .map_err(std::io::Error::other)?,
    )
    .ok_or_else(|| std::io::Error::other("no display outputs were selected"))?;

    eprintln!("{:?}", output);

    wf_info_overlay::run(RewardOverlay {
        output_name: output.name,
        output_size: Some(output.size),
        duration: Some(Duration::from_secs(5)),
        rewards: vec![
            RewardOverlayEntry::name_only("Forma Blueprint")
                .with_platinum(8)
                .with_ducats(0)
                .with_volume(172),
            RewardOverlayEntry::name_only("Braton Prime Receiver")
                .with_platinum(42)
                .with_ducats(45)
                .with_volume(18)
                .with_vaulted(true),
            RewardOverlayEntry::name_only("Paris Prime String")
                .with_platinum(15)
                .with_ducats(25)
                .with_volume(36),
            {
                let mut reward = RewardOverlayEntry::name_only("Akbronco Prime Link")
                    .with_platinum(24)
                    .with_ducats(45)
                    .with_volume(7);
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

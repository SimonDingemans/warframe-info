use crate::{RewardCardEntry, RewardHighlight};

pub fn reward_cards_from_scan_output(output: &info_core::ScanOutput) -> Vec<RewardCardEntry> {
    let limit = match output.kind {
        info_core::ScanKind::Reward => 4,
        info_core::ScanKind::Inventory => output.items.len(),
    };

    output
        .items
        .iter()
        .take(limit)
        .map(reward_card_from_item)
        .collect()
}

pub fn reward_card_from_item(item: &info_core::WarframeItem) -> RewardCardEntry {
    RewardCardEntry {
        name: item.drop_name.clone(),
        platinum: Some(item.platinum_rounded()),
        ducats: item.ducats,
        volume: Some(item.volume),
        vaulted: item.vaulted,
        mastered: false,
        owned_count: None,
        required_count: None,
        highlight: RewardHighlight::None,
    }
}

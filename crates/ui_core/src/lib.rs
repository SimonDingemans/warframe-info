mod reward_card;

#[cfg(feature = "scan")]
mod scan;

#[cfg(feature = "iced-ui")]
mod iced_ui;

pub use reward_card::{
    best_platinum_reward_index, reward_is_best_platinum, RewardCardEntry, RewardHighlight,
    REWARD_CARD_HEIGHT, REWARD_CARD_SPACING, REWARD_CARD_WIDTH, REWARD_OVERLAY_PADDING,
};

#[cfg(feature = "scan")]
pub use scan::{reward_card_from_item, reward_cards_from_scan_output};

#[cfg(feature = "iced-ui")]
pub use iced_ui::{
    decode_reward_icon, reward_card, reward_cards_row, reward_icon_handle, RewardCardAssets,
    DUCAT_ICON_BYTES, PLATINUM_ICON_BYTES,
};

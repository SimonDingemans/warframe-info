pub const REWARD_CARD_WIDTH: u32 = 180;
pub const REWARD_CARD_HEIGHT: u32 = 154;
pub const REWARD_CARD_SPACING: u32 = 10;
pub const REWARD_OVERLAY_PADDING: u32 = 18;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardCardEntry {
    pub name: String,
    pub platinum: Option<u32>,
    pub ducats: Option<u32>,
    pub vaulted: bool,
    pub highlight: RewardHighlight,
}

impl RewardCardEntry {
    pub fn name_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            platinum: None,
            ducats: None,
            vaulted: false,
            highlight: RewardHighlight::None,
        }
    }

    pub fn with_platinum(mut self, platinum: u32) -> Self {
        self.platinum = Some(platinum);
        self
    }

    pub fn with_ducats(mut self, ducats: u32) -> Self {
        self.ducats = Some(ducats);
        self
    }

    pub fn with_vaulted(mut self, vaulted: bool) -> Self {
        self.vaulted = vaulted;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RewardHighlight {
    #[default]
    None,
    BestPlatinum,
    BestDucats,
}

pub fn best_platinum_reward_index(rewards: &[RewardCardEntry]) -> Option<usize> {
    rewards
        .iter()
        .enumerate()
        .filter_map(|(index, reward)| reward.platinum.map(|platinum| (index, platinum)))
        .fold(None, |best, candidate| match best {
            Some((_, best_platinum)) if best_platinum >= candidate.1 => best,
            _ => Some(candidate),
        })
        .map(|(index, _)| index)
}

pub fn reward_is_best_platinum(
    index: usize,
    reward: &RewardCardEntry,
    best_platinum: Option<usize>,
) -> bool {
    reward.highlight == RewardHighlight::BestPlatinum || best_platinum == Some(index)
}

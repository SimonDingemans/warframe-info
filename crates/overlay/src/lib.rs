mod display;

pub use display::{
    DisplayBackend, DisplayOutput, DisplayOutputsFuture, DisplayResult, DynDisplayBackend,
};

pub const PLATINUM_ICON_BYTES: &[u8] = include_bytes!("../assets/src/PlatinumLarge.png");
pub const DUCAT_ICON_BYTES: &[u8] = include_bytes!("../assets/src/OrokinDucats.png");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardOverlayEntry {
    pub name: String,
    pub platinum: Option<u32>,
    pub ducats: Option<u32>,
    pub volume: Option<u32>,
    pub vaulted: bool,
    pub mastered: bool,
    pub owned_count: Option<u32>,
    pub required_count: Option<u32>,
    pub highlight: RewardHighlight,
}

impl RewardOverlayEntry {
    pub fn name_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            platinum: None,
            ducats: None,
            volume: None,
            vaulted: false,
            mastered: false,
            owned_count: None,
            required_count: None,
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

    pub fn with_volume(mut self, volume: u32) -> Self {
        self.volume = Some(volume);
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
    Needed,
}

#[derive(Clone, Debug)]
pub struct RewardOverlay {
    pub output_name: Option<String>,
    pub output_size: Option<(u32, u32)>,
    pub duration: Option<std::time::Duration>,
    pub rewards: Vec<RewardOverlayEntry>,
}

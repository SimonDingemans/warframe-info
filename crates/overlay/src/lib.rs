mod display;

pub use display::{
    DisplayBackend, DisplayOutput, DisplayOutputsFuture, DisplayResult, DynDisplayBackend,
};

#[derive(Clone, Debug)]
pub struct RewardOverlay {
    pub output_name: Option<String>,
    pub output_size: Option<(u32, u32)>,
    pub duration: Option<std::time::Duration>,
    pub rewards: Vec<ui_core::RewardCardEntry>,
}

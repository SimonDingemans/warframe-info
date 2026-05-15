pub async fn display_outputs() -> Result<Vec<overlay::DisplayOutput>, String> {
    super::imp::display_outputs().await
}

pub fn reset_display_restore_token() -> Result<(), String> {
    super::imp::reset_display_restore_token()
}

pub fn run(overlay: overlay::RewardOverlay) -> Result<(), String> {
    super::imp::run_reward_overlay(overlay)
}

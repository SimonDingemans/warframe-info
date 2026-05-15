use crate::{CropResult, ImageSize, PixelRect, RatioRect, ScreenCrop};

#[derive(Debug, Clone)]
pub struct RewardScreenCrop {
    crop: RatioRect,
}

impl Default for RewardScreenCrop {
    fn default() -> Self {
        Self {
            crop: RatioRect::new(0.2484375, 0.20694445, 0.51953125, 0.2337963),
        }
    }
}

impl ScreenCrop for RewardScreenCrop {
    fn crop_rect(&self, source_size: ImageSize) -> CropResult<PixelRect> {
        self.crop.to_pixel_rect(source_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reward_screen_crop_uses_reward_card_strip() {
        let crop = RewardScreenCrop::default()
            .crop_rect(ImageSize {
                width: 3840,
                height: 2160,
            })
            .expect("crop should be valid");

        assert_eq!(
            crop,
            PixelRect {
                x: 954,
                y: 447,
                width: 1995,
                height: 505,
            }
        );
    }
}

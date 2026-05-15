use crate::{CropResult, ImageSize, PixelRect, RatioRect, ScreenCrop};

#[derive(Debug, Clone)]
pub struct InventoryCrop {
    crop: RatioRect,
}

impl Default for InventoryCrop {
    fn default() -> Self {
        Self {
            crop: RatioRect::new(0.037715517, 0.20561941, 0.6508621, 0.7943806),
        }
    }
}

impl ScreenCrop for InventoryCrop {
    fn crop_rect(&self, source_size: ImageSize) -> CropResult<PixelRect> {
        self.crop.to_pixel_rect(source_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_crop_uses_default_item_grid() {
        let crop = InventoryCrop::default()
            .crop_rect(ImageSize {
                width: 2784,
                height: 1566,
            })
            .expect("crop should be valid");

        assert_eq!(
            crop,
            PixelRect {
                x: 105,
                y: 322,
                width: 1812,
                height: 1244,
            }
        );
    }
}

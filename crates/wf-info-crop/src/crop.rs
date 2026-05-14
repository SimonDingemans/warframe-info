use image::DynamicImage;

use crate::{CropResult, ImageSize, PixelRect};

#[derive(Debug, Clone)]
pub struct CroppedImage {
    pub image: DynamicImage,
    pub source_size: ImageSize,
    pub crop: PixelRect,
    pub kind: ScreenCropKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenCropKind {
    Inventory,
    RewardScreen,
}

pub trait ScreenCrop: Send + Sync {
    fn kind(&self) -> ScreenCropKind;
    fn crop_rect(&self, source_size: ImageSize) -> CropResult<PixelRect>;

    fn crop_image(&self, image: &DynamicImage) -> CropResult<CroppedImage> {
        let source_size = ImageSize::from_image(image);
        let crop = self.crop_rect(source_size)?;
        let image = image.crop_imm(crop.x, crop.y, crop.width, crop.height);

        Ok(CroppedImage {
            image,
            source_size,
            crop,
            kind: self.kind(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RatioRect, RewardScreenCrop};

    #[test]
    fn crop_image_returns_metadata_and_cropped_image() {
        let image = DynamicImage::new_rgba8(100, 50);
        let cropper = RewardScreenCrop::default().with_crop(RatioRect::new(0.25, 0.2, 0.5, 0.4));

        let cropped = cropper.crop_image(&image).expect("crop should be valid");

        assert_eq!(cropped.kind, ScreenCropKind::RewardScreen);
        assert_eq!(
            cropped.crop,
            PixelRect {
                x: 25,
                y: 10,
                width: 50,
                height: 20,
            }
        );
        assert_eq!(cropped.image.width(), 50);
        assert_eq!(cropped.image.height(), 20);
    }
}

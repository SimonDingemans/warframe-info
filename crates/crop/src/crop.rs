use image::DynamicImage;

use crate::{CropResult, ImageSize, PixelRect};

#[derive(Debug, Clone)]
pub struct CroppedImage {
    pub image: DynamicImage,
    pub source_size: ImageSize,
}

pub trait ScreenCrop: Send + Sync {
    fn crop_rect(&self, source_size: ImageSize) -> CropResult<PixelRect>;

    fn crop_image(&self, image: &DynamicImage) -> CropResult<CroppedImage> {
        let source_size = ImageSize::from_image(image);
        let crop = self.crop_rect(source_size)?;
        let image = image.crop_imm(crop.x, crop.y, crop.width, crop.height);

        Ok(CroppedImage { image, source_size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCrop;

    impl ScreenCrop for TestCrop {
        fn crop_rect(&self, _source_size: ImageSize) -> CropResult<PixelRect> {
            Ok(PixelRect {
                x: 25,
                y: 10,
                width: 50,
                height: 20,
            })
        }
    }

    #[test]
    fn crop_image_returns_source_metadata_and_cropped_image() {
        let image = DynamicImage::new_rgba8(100, 50);
        let cropper = TestCrop;

        let cropped = cropper.crop_image(&image).expect("crop should be valid");

        assert_eq!(
            cropped.source_size,
            ImageSize {
                width: 100,
                height: 50
            }
        );
        assert_eq!(cropped.image.width(), 50);
        assert_eq!(cropped.image.height(), 20);
    }
}

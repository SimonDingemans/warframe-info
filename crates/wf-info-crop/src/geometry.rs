use image::DynamicImage;

use crate::{CropError, CropResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl ImageSize {
    pub fn from_image(image: &DynamicImage) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatioRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RatioRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn to_pixel_rect(self, source_size: ImageSize) -> CropResult<PixelRect> {
        if source_size.width == 0 || source_size.height == 0 {
            return Err(CropError::EmptySourceImage);
        }

        let x_min = self.x.clamp(0.0, 1.0);
        let y_min = self.y.clamp(0.0, 1.0);
        let x_max = (self.x + self.width).clamp(0.0, 1.0);
        let y_max = (self.y + self.height).clamp(0.0, 1.0);

        let x = ratio_to_pixel(x_min, source_size.width);
        let y = ratio_to_pixel(y_min, source_size.height);
        let right = ratio_to_pixel(x_max, source_size.width);
        let bottom = ratio_to_pixel(y_max, source_size.height);
        let width = right.saturating_sub(x);
        let height = bottom.saturating_sub(y);

        if width == 0 || height == 0 {
            return Err(CropError::EmptyCrop {
                source_size,
                ratio: self,
            });
        }

        Ok(PixelRect {
            x,
            y,
            width,
            height,
        })
    }
}

fn ratio_to_pixel(ratio: f32, size: u32) -> u32 {
    (ratio * size as f32).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_crop_returns_error() {
        let err = RatioRect::new(1.0, 1.0, 0.0, 0.0)
            .to_pixel_rect(ImageSize {
                width: 100,
                height: 100,
            })
            .expect_err("empty crop should fail");

        assert!(matches!(err, CropError::EmptyCrop { .. }));
    }
}

use thiserror::Error;

use crate::{ImageSize, RatioRect};

pub type CropResult<T> = Result<T, CropError>;

#[derive(Debug, Error)]
pub enum CropError {
    #[error("source image is empty")]
    EmptySourceImage,

    #[error(
        "crop ratio produced an empty crop for {source_size:?}: x={x}, y={y}, width={width}, height={height}",
        x = ratio.x,
        y = ratio.y,
        width = ratio.width,
        height = ratio.height
    )]
    EmptyCrop {
        source_size: ImageSize,
        ratio: RatioRect,
    },
}

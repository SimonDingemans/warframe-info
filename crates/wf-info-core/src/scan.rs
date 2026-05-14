use image::DynamicImage;
use thiserror::Error;
use wf_info_crop::{InventoryCrop, RewardScreenCrop, ScreenCrop};
use wf_info_ocr::{
    layouts::{InventoryGridLayout, RewardScreenLayout},
    load_ocr_engine,
    pipeline::ItemPipeline,
    text::WarframeTextNormalizer,
};

use crate::item_database::{ItemDatabase, WarframeItem};

pub type ScanResult<T> = Result<T, ScanError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScanKind {
    Reward,
    Inventory,
}

impl ScanKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Reward => "Reward",
            Self::Inventory => "Inventory",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScanOutput {
    pub kind: ScanKind,
    pub source_width: u32,
    pub source_height: u32,
    pub cropped_width: u32,
    pub cropped_height: u32,
    pub text_block_count: usize,
    pub items: Vec<WarframeItem>,
}

pub fn scan_image_with_item_database(
    kind: ScanKind,
    screenshot: &DynamicImage,
    database: &ItemDatabase,
) -> ScanResult<ScanOutput> {
    match kind {
        ScanKind::Reward => scan_reward_image(screenshot, database),
        ScanKind::Inventory => scan_inventory_image(screenshot, database),
    }
}

fn scan_reward_image(screenshot: &DynamicImage, database: &ItemDatabase) -> ScanResult<ScanOutput> {
    let cropped = RewardScreenCrop::default().crop_image(screenshot)?;
    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine().map_err(|source| ScanError::Ocr {
        message: source.to_string(),
    })?;
    let output = pipeline
        .run(&mut ocr, &cropped.image, &RewardScreenLayout::default())
        .map_err(|source| ScanError::Ocr {
            message: source.to_string(),
        })?;

    Ok(ScanOutput {
        kind: ScanKind::Reward,
        source_width: screenshot.width(),
        source_height: screenshot.height(),
        cropped_width: cropped.image.width(),
        cropped_height: cropped.image.height(),
        text_block_count: output.text_blocks.len(),
        items: database.find_items(output.items.iter().map(String::as_str)),
    })
}

fn scan_inventory_image(
    screenshot: &DynamicImage,
    database: &ItemDatabase,
) -> ScanResult<ScanOutput> {
    let cropped = InventoryCrop::default().crop_image(screenshot)?;
    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine().map_err(|source| ScanError::Ocr {
        message: source.to_string(),
    })?;
    let output = pipeline
        .run(&mut ocr, &cropped.image, &InventoryGridLayout::new(6))
        .map_err(|source| ScanError::Ocr {
            message: source.to_string(),
        })?;

    Ok(ScanOutput {
        kind: ScanKind::Inventory,
        source_width: screenshot.width(),
        source_height: screenshot.height(),
        cropped_width: cropped.image.width(),
        cropped_height: cropped.image.height(),
        text_block_count: output.text_blocks.len(),
        items: database.find_items(output.items.iter().map(String::as_str)),
    })
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("failed to crop screenshot")]
    Crop(#[from] wf_info_crop::CropError),

    #[error("OCR pipeline failed: {message}")]
    Ocr { message: String },
}

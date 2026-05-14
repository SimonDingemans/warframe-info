use image::DynamicImage;
use std::error::Error;

pub type PipelineResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageSize {
    pub width: f32,
    pub height: f32,
}

impl ImageSize {
    pub fn from_image(image: &DynamicImage) -> Self {
        Self {
            width: image.width() as f32,
            height: image.height() as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextBounds {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl TextBounds {
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn center_x(&self) -> f32 {
        (self.x_min + self.x_max) / 2.0
    }

    pub fn center_y(&self) -> f32 {
        (self.y_min + self.y_max) / 2.0
    }

    pub fn crop_from(&self, image: &DynamicImage) -> Option<DynamicImage> {
        let image_width = image.width() as f32;
        let image_height = image.height() as f32;

        let x_min = self.x_min.max(0.0).min(image_width).floor() as u32;
        let y_min = self.y_min.max(0.0).min(image_height).floor() as u32;
        let x_max = self.x_max.max(0.0).min(image_width).ceil() as u32;
        let y_max = self.y_max.max(0.0).min(image_height).ceil() as u32;

        let width = x_max.saturating_sub(x_min);
        let height = y_max.saturating_sub(y_min);
        if width == 0 || height == 0 {
            return None;
        }

        Some(image.crop_imm(x_min, y_min, width, height))
    }
}

#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
    pub score: f32,
    pub bounds: TextBounds,
}

#[derive(Debug, Clone)]
pub struct RecognizedText {
    pub text: String,
    pub score: f32,
}

pub trait TextOcrEngine {
    fn detect_text_bounds(&mut self, image: &DynamicImage) -> PipelineResult<Vec<TextBounds>>;
    fn recognize_text(
        &mut self,
        image_crop: &DynamicImage,
    ) -> PipelineResult<Option<RecognizedText>>;
}

pub trait TextNormalizer {
    fn normalize(&self, text: &str) -> Option<String>;
}

pub trait ItemLayout {
    type Item;

    fn should_recover_stacked_text_blocks(&self) -> bool {
        false
    }

    fn accepts_text_bounds(&self, _bounds: &TextBounds, _image_size: ImageSize) -> bool {
        true
    }

    fn group_text_blocks(&self, blocks: &[TextBlock], image_size: ImageSize) -> Vec<Self::Item>;
}

#[derive(Debug, Clone)]
pub struct PipelineOutput<T> {
    pub text_blocks: Vec<TextBlock>,
    pub items: Vec<T>,
}

pub struct ItemPipeline<N> {
    min_text_score: f32,
    normalizer: N,
}

impl<N> ItemPipeline<N>
where
    N: TextNormalizer,
{
    pub fn new(normalizer: N) -> Self {
        Self {
            min_text_score: 0.75,
            normalizer,
        }
    }

    pub fn with_min_text_score(mut self, min_text_score: f32) -> Self {
        self.min_text_score = min_text_score;
        self
    }

    pub fn run<E, L>(
        &self,
        ocr: &mut E,
        cropped_image: &DynamicImage,
        layout: &L,
    ) -> PipelineResult<PipelineOutput<L::Item>>
    where
        E: TextOcrEngine,
        L: ItemLayout,
    {
        let image_size = ImageSize::from_image(cropped_image);
        let text_bounds = ocr.detect_text_bounds(cropped_image)?;
        let mut text_blocks = Vec::new();

        for bounds in text_bounds {
            if !layout.accepts_text_bounds(&bounds, image_size) {
                continue;
            }

            let Some(text_crop) = bounds.crop_from(cropped_image) else {
                continue;
            };

            let Some(recognized) = ocr.recognize_text(&text_crop)? else {
                continue;
            };

            if recognized.score < self.min_text_score {
                continue;
            }

            let Some(text) = self.normalizer.normalize(&recognized.text) else {
                continue;
            };

            text_blocks.push(TextBlock {
                text,
                score: recognized.score,
                bounds,
            });
        }

        if layout.should_recover_stacked_text_blocks() {
            let recovered_blocks =
                self.recover_stacked_text_blocks(ocr, cropped_image, &text_blocks)?;
            text_blocks.extend(recovered_blocks);
        }

        let items = layout.group_text_blocks(&text_blocks, image_size);
        Ok(PipelineOutput { text_blocks, items })
    }

    fn recover_stacked_text_blocks<E>(
        &self,
        ocr: &mut E,
        cropped_image: &DynamicImage,
        text_blocks: &[TextBlock],
    ) -> PipelineResult<Vec<TextBlock>>
    where
        E: TextOcrEngine,
    {
        let mut recovered_blocks = Vec::new();

        for block in text_blocks {
            if !is_single_word(&block.text)
                || has_stacked_prefix(block, text_blocks)
                || has_same_line_neighbor(block, text_blocks)
            {
                continue;
            }

            let height = block.bounds.height();
            let width = block.bounds.width();
            let recovery_bounds = TextBounds {
                x_min: (block.bounds.x_min - width * 0.15).max(0.0),
                y_min: (block.bounds.y_min - height * 2.5).max(0.0),
                x_max: (block.bounds.x_max + width * 0.15).min(cropped_image.width() as f32),
                y_max: block.bounds.y_min,
            };

            if recovery_bounds.height() < height * 0.75 || recovery_bounds.width() < height {
                continue;
            }

            let Some(text_crop) = recovery_bounds.crop_from(cropped_image) else {
                continue;
            };

            let Some(recognized) = ocr.recognize_text(&text_crop)? else {
                continue;
            };

            if recognized.score < self.min_text_score {
                continue;
            }

            let Some(text) = self.normalizer.normalize(&recognized.text) else {
                continue;
            };

            if text == block.text
                || text_blocks.iter().any(|existing| {
                    existing.text == text
                        && horizontal_overlap_ratio(&existing.bounds, &recovery_bounds) >= 0.25
                        && vertical_overlap_ratio(&existing.bounds, &recovery_bounds) >= 0.25
                })
            {
                continue;
            }

            recovered_blocks.push(TextBlock {
                text,
                score: recognized.score,
                bounds: recovery_bounds,
            });
        }

        Ok(recovered_blocks)
    }
}

fn is_single_word(text: &str) -> bool {
    text.split_whitespace().count() == 1
}

fn has_stacked_prefix(block: &TextBlock, blocks: &[TextBlock]) -> bool {
    blocks.iter().any(|candidate| {
        !std::ptr::eq(candidate, block)
            && candidate.bounds.center_y() < block.bounds.center_y()
            && vertical_gap(&candidate.bounds, &block.bounds) <= block.bounds.height() * 2.0
            && horizontal_overlap_ratio(&candidate.bounds, &block.bounds) >= 0.25
    })
}

fn has_same_line_neighbor(block: &TextBlock, blocks: &[TextBlock]) -> bool {
    blocks.iter().any(|candidate| {
        !std::ptr::eq(candidate, block)
            && vertical_overlap_ratio(&candidate.bounds, &block.bounds) >= 0.25
            && horizontal_gap(&candidate.bounds, &block.bounds) <= block.bounds.height() * 2.0
    })
}

fn vertical_gap(above: &TextBounds, below: &TextBounds) -> f32 {
    (below.y_min - above.y_max).max(0.0)
}

fn horizontal_gap(a: &TextBounds, b: &TextBounds) -> f32 {
    if a.x_max < b.x_min {
        b.x_min - a.x_max
    } else if b.x_max < a.x_min {
        a.x_min - b.x_max
    } else {
        0.0
    }
}

fn horizontal_overlap_ratio(a: &TextBounds, b: &TextBounds) -> f32 {
    let overlap = a.x_max.min(b.x_max) - a.x_min.max(b.x_min);
    if overlap <= 0.0 {
        return 0.0;
    }

    overlap / a.width().min(b.width()).max(f32::EPSILON)
}

fn vertical_overlap_ratio(a: &TextBounds, b: &TextBounds) -> f32 {
    let overlap = a.y_max.min(b.y_max) - a.y_min.max(b.y_min);
    if overlap <= 0.0 {
        return 0.0;
    }

    overlap / a.height().min(b.height()).max(f32::EPSILON)
}

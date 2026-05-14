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

        let items = layout.group_text_blocks(&text_blocks, image_size);
        Ok(PipelineOutput { text_blocks, items })
    }
}

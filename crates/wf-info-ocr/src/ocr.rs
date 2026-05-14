use crate::pipeline::{PipelineResult, RecognizedText, TextBounds, TextOcrEngine};
use image::{imageops::FilterType, DynamicImage, GrayImage, Luma};
use imageproc::contours::find_contours;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};

const DETECTOR_SIZE: u32 = 960;

pub struct PaddleOcrEngine {
    detector: Session,
    recognizer: Session,
    dictionary: Vec<String>,
}

impl PaddleOcrEngine {
    pub fn from_files(
        detector_model_path: &str,
        recognizer_model_path: &str,
    ) -> PipelineResult<Self> {
        let detector = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(detector_model_path)?;

        let recognizer = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(recognizer_model_path)?;

        Ok(Self {
            detector,
            recognizer,
            dictionary: load_dictionary(),
        })
    }
}

impl TextOcrEngine for PaddleOcrEngine {
    fn detect_text_bounds(&mut self, image: &DynamicImage) -> PipelineResult<Vec<TextBounds>> {
        run_detector(&mut self.detector, image)
    }

    fn recognize_text(
        &mut self,
        image_crop: &DynamicImage,
    ) -> PipelineResult<Option<RecognizedText>> {
        recognize_text(&mut self.recognizer, image_crop, &self.dictionary)
    }
}

fn load_dictionary() -> Vec<String> {
    let mut dict = vec!["<blank>".to_string()];

    for c in '0'..='9' {
        dict.push(c.to_string());
    }
    for c in 'A'..='Z' {
        dict.push(c.to_string());
    }
    for c in 'a'..='z' {
        dict.push(c.to_string());
    }

    dict.push(" ".to_string());
    dict
}

fn run_detector(session: &mut Session, image: &DynamicImage) -> PipelineResult<Vec<TextBounds>> {
    let orig_w = image.width() as f32;
    let orig_h = image.height() as f32;
    let resized = image
        .resize_exact(DETECTOR_SIZE, DETECTOR_SIZE, FilterType::Triangle)
        .to_rgb8();
    let detector_size = DETECTOR_SIZE as usize;
    let mut input_data = vec![0.0; 3 * detector_size * detector_size];

    let mean = [0.485, 0.456, 0.406];
    let std = [0.229, 0.224, 0.225];

    for (x, y, pixel) in resized.enumerate_pixels() {
        for c in 0..3 {
            let val = pixel[c] as f32 / 255.0;
            input_data
                [c * detector_size * detector_size + y as usize * detector_size + x as usize] =
                (val - mean[c]) / std[c];
        }
    }

    let input_tensor = Tensor::from_array(([1usize, 3, detector_size, detector_size], input_data))?;
    let outputs = session.run(ort::inputs![input_tensor])?;
    let view = outputs[0].try_extract_array::<f32>()?;

    let mut thresh_img = GrayImage::new(DETECTOR_SIZE, DETECTOR_SIZE);
    for y in 0..DETECTOR_SIZE {
        for x in 0..DETECTOR_SIZE {
            let prob = view[[0, 0, y as usize, x as usize]];
            if prob > 0.3 {
                thresh_img.put_pixel(x, y, Luma([255]));
            }
        }
    }

    let contours = find_contours::<i32>(&thresh_img);
    let scale_x = orig_w / DETECTOR_SIZE as f32;
    let scale_y = orig_h / DETECTOR_SIZE as f32;
    let mut bounds = Vec::new();

    for contour in contours {
        if contour.points.is_empty() {
            continue;
        }

        let mut min_x = DETECTOR_SIZE as f32;
        let mut max_x = 0.0;
        let mut min_y = DETECTOR_SIZE as f32;
        let mut max_y = 0.0;

        for point in contour.points {
            let px = point.x as f32;
            let py = point.y as f32;
            if px < min_x {
                min_x = px;
            }
            if px > max_x {
                max_x = px;
            }
            if py < min_y {
                min_y = py;
            }
            if py > max_y {
                max_y = py;
            }
        }

        let height = max_y - min_y;
        let width = max_x - min_x;
        if height < 5.0 || width < 5.0 {
            continue;
        }

        let expand_y = height * 0.3;
        let expand_x = width * 0.1;

        bounds.push(TextBounds {
            x_min: (min_x - expand_x).max(0.0) * scale_x,
            y_min: (min_y - expand_y).max(0.0) * scale_y,
            x_max: (max_x + expand_x).min(DETECTOR_SIZE as f32) * scale_x,
            y_max: (max_y + expand_y).min(DETECTOR_SIZE as f32) * scale_y,
        });
    }

    Ok(bounds)
}

fn recognize_text(
    session: &mut Session,
    image_crop: &DynamicImage,
    dictionary: &[String],
) -> PipelineResult<Option<RecognizedText>> {
    let aspect_ratio = image_crop.width() as f32 / image_crop.height() as f32;
    let target_height = 48;
    let mut target_width = (target_height as f32 * aspect_ratio).ceil() as u32;
    target_width = target_width.clamp(10, 800);

    let resized = image_crop
        .resize_exact(target_width, target_height, FilterType::Triangle)
        .to_rgb8();
    let target_height = target_height as usize;
    let target_width = target_width as usize;
    let channel_size = target_height * target_width;
    let mut input_data = vec![0.0; 3 * channel_size];

    for (x, y, pixel) in resized.enumerate_pixels() {
        let r = (pixel[0] as f32 / 255.0 - 0.5) / 0.5;
        let g = (pixel[1] as f32 / 255.0 - 0.5) / 0.5;
        let b = (pixel[2] as f32 / 255.0 - 0.5) / 0.5;
        let pixel_offset = y as usize * target_width + x as usize;
        input_data[pixel_offset] = r;
        input_data[channel_size + pixel_offset] = g;
        input_data[2 * channel_size + pixel_offset] = b;
    }

    let input_tensor = Tensor::from_array(([1usize, 3, target_height, target_width], input_data))?;
    let outputs = session.run(ort::inputs![input_tensor])?;
    let view = outputs[0].try_extract_array::<f32>()?;
    let shape = view.shape();
    let seq_len = shape[1];
    let dict_size = shape[2];

    let mut text = String::new();
    let mut last_index = 0;
    let mut total_score = 0.0;
    let mut valid_chars = 0;

    for i in 0..seq_len {
        let mut max_prob = 0.0;
        let mut max_idx = 0;
        for j in 0..dict_size {
            let prob = view[[0, i, j]];
            if prob > max_prob {
                max_prob = prob;
                max_idx = j;
            }
        }
        if max_idx != 0 && max_idx != last_index && max_idx < dictionary.len() {
            text.push_str(&dictionary[max_idx]);
            total_score += max_prob;
            valid_chars += 1;
        }
        last_index = max_idx;
    }

    if valid_chars == 0 {
        return Ok(None);
    }

    Ok(Some(RecognizedText {
        text,
        score: total_score / valid_chars as f32,
    }))
}

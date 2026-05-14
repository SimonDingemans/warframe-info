use image::{imageops::FilterType, DynamicImage, GrayImage, Luma};
use imageproc::contours::find_contours;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::collections::BTreeMap;

const TOTAL_COLUMNS: usize = 6;
const SAFE_TOP_RATIO: f32 = 0.12;
const SAFE_BOTTOM_RATIO: f32 = 0.85;
const LINE_Y_TOLERANCE: f32 = 30.0;
const DETECTOR_SIZE: u32 = 960; // Paddle typically resizes to a max of 960 (multiple of 32)

#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
    pub score: f32,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub row: Option<usize>,
    pub col: Option<usize>,
}

impl TextBlock {
    pub fn center_x(&self) -> f32 {
        (self.x_min + self.x_max) / 2.0
    }
    pub fn center_y(&self) -> f32 {
        (self.y_min + self.y_max) / 2.0
    }
}

fn load_dictionary() -> Vec<String> {
    let mut dict = vec!["<blank>".to_string()];
    
    // Add 0-9
    for c in '0'..='9' {
        dict.push(c.to_string());
    }
    // Add A-Z
    for c in 'A'..='Z' {
        dict.push(c.to_string());
    }
    // Add a-z
    for c in 'a'..='z' {
        dict.push(c.to_string());
    }
    
    // Add space at the end
    dict.push(" ".to_string());
    
    dict
}

fn clean_text(text: &str) -> Option<String> {
    let mut cleaned = text.trim().to_string();

    if let Some(prime_start) = cleaned.find("Prime") {
        if cleaned[..prime_start].chars().all(|c| c.is_ascii_lowercase()) {
            cleaned.replace_range(..prime_start, "");
        }
    }
    if cleaned.is_empty() { return None; }

    if cleaned.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase()) {
        let mut chars = cleaned.chars();
        cleaned = match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        };
    }

	let mut spaced_text = String::new();
    let mut chars = cleaned.chars().peekable();
    
    while let Some(c) = chars.next() {
        spaced_text.push(c);
        
        // Peek at the next character
        if let Some(&next_c) = chars.peek() {
            // If current is lowercase and next is uppercase, add a space
            if c.is_ascii_lowercase() && next_c.is_ascii_uppercase() {
                spaced_text.push(' ');
            }
        }
    }

    Some(cleaned)
}

/// 2. Run Detector to find Bounding Boxes
fn run_detector(session: &mut Session, img: &DynamicImage) -> ort::Result<Vec<(f32, f32, f32, f32)>> {
    let orig_w = img.width() as f32;
    let orig_h = img.height() as f32;

    // Resize image for detector
    let resized = img.resize_exact(DETECTOR_SIZE, DETECTOR_SIZE, FilterType::Triangle).to_rgb8();
    let detector_size = DETECTOR_SIZE as usize;
    let mut input_data = vec![0.0; 3 * detector_size * detector_size];

    // ImageNet Normalization used by Paddle Det
    let mean = [0.485, 0.456, 0.406];
    let std = [0.229, 0.224, 0.225];

    for (x, y, pixel) in resized.enumerate_pixels() {
        for c in 0..3 {
            let val = pixel[c] as f32 / 255.0;
            input_data[c * detector_size * detector_size + y as usize * detector_size + x as usize] =
                (val - mean[c]) / std[c];
        }
    }

    let input_tensor = Tensor::from_array(([1usize, 3, detector_size, detector_size], input_data))?;
    let outputs = session.run(ort::inputs![input_tensor])?;
    let view = outputs[0].try_extract_array::<f32>()?;

    // Threshold the heatmap to a B&W image
    let mut thresh_img = GrayImage::new(DETECTOR_SIZE, DETECTOR_SIZE);
    for y in 0..DETECTOR_SIZE {
        for x in 0..DETECTOR_SIZE {
            // Heatmap is shape [1, 1, 960, 960]
            let prob = view[[0, 0, y as usize, x as usize]];
            if prob > 0.3 {
                thresh_img.put_pixel(x, y, Luma([255]));
            }
        }
    }

    // Find contours using imageproc
    let contours = find_contours::<i32>(&thresh_img);
    let mut boxes = Vec::new();

    let scale_x = orig_w / DETECTOR_SIZE as f32;
    let scale_y = orig_h / DETECTOR_SIZE as f32;

    for contour in contours {
        if contour.points.is_empty() { continue; }
        
        let mut min_x = DETECTOR_SIZE as f32;
        let mut max_x = 0.0;
        let mut min_y = DETECTOR_SIZE as f32;
        let mut max_y = 0.0;

        for p in contour.points {
            let px = p.x as f32;
            let py = p.y as f32;
            if px < min_x { min_x = px; }
            if px > max_x { max_x = px; }
            if py < min_y { min_y = py; }
            if py > max_y { max_y = py; }
        }

        // DBNet predicts shrunk regions. We expand the AABB to capture the full letters.
        let height = max_y - min_y;
        let width = max_x - min_x;
        
        // Skip tiny noise artifacts
        if height < 5.0 || width < 5.0 { continue; }

        let expand_y = height * 0.3;
        let expand_x = width * 0.1;

        let expanded_min_x = (min_x - expand_x).max(0.0) * scale_x;
        let expanded_min_y = (min_y - expand_y).max(0.0) * scale_y;
        let expanded_max_x = (max_x + expand_x).min(DETECTOR_SIZE as f32) * scale_x;
        let expanded_max_y = (max_y + expand_y).min(DETECTOR_SIZE as f32) * scale_y;

        boxes.push((expanded_min_x, expanded_min_y, expanded_max_x, expanded_max_y));
    }

    Ok(boxes)
}

/// 3. Run Recognizer and Decode Text
fn recognize_text(
    session: &mut Session,
    image_crop: &DynamicImage,
    dictionary: &[String],
) -> ort::Result<Option<(String, f32)>> {
    let aspect_ratio = image_crop.width() as f32 / image_crop.height() as f32;
    let target_height = 48;
    let mut target_width = (target_height as f32 * aspect_ratio).ceil() as u32;
    
    // Width must be at least small enough to not break tensor limits, usually safe around 320 max
    target_width = target_width.clamp(10, 800);
    
    let resized = image_crop.resize_exact(target_width, target_height, FilterType::Triangle).to_rgb8();
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
        if max_idx != 0 && max_idx != last_index {
            if max_idx < dictionary.len() {
                text.push_str(&dictionary[max_idx]);
                total_score += max_prob;
                valid_chars += 1;
            }
        }
        last_index = max_idx;
    }

    if valid_chars == 0 { return Ok(None); }
    Ok(Some((text, total_score / valid_chars as f32)))
}

// ... [Grid Binning Functions (assign_grid_positions, sort_cell_blocks, group_inventory_items) go here exactly as written previously] ...
fn assign_grid_positions(blocks: &mut [TextBlock], image_width: f32, image_height: f32) {
    let column_width = image_width / TOTAL_COLUMNS as f32;
    for block in blocks.iter_mut() {
        let col = (block.center_x() / column_width) as usize;
        block.col = Some(col.clamp(0, TOTAL_COLUMNS.saturating_sub(1)));
    }
    let row_gap_threshold = image_height * 0.12;
    let mut row_centers: Vec<Vec<f32>> = Vec::new();
    let mut sorted_y: Vec<f32> = blocks.iter().map(|b| b.center_y()).collect();
    sorted_y.sort_by(|a, b| a.total_cmp(b));

    for center_y in sorted_y {
        if row_centers.is_empty() {
            row_centers.push(vec![center_y]);
        } else {
            let last_row = row_centers.last_mut().unwrap();
            let last_y = *last_row.last().unwrap();
            if center_y - last_y > row_gap_threshold {
                row_centers.push(vec![center_y]);
            } else {
                last_row.push(center_y);
            }
        }
    }
    let row_anchors: Vec<f32> = row_centers.iter().map(|row| row.iter().sum::<f32>() / row.len() as f32).collect();
    for block in blocks.iter_mut() {
        let center_y = block.center_y();
        let (closest_idx, _) = row_anchors.iter().enumerate()
            .min_by(|(_, a), (_, b)| (center_y - *a).abs().total_cmp(&(center_y - *b).abs())).unwrap();
        block.row = Some(closest_idx);
    }
}

fn average_line_y(line: &[TextBlock]) -> f32 {
    line.iter().map(|b| b.center_y()).sum::<f32>() / line.len() as f32
}

fn sort_cell_blocks(mut blocks: Vec<TextBlock>) -> Vec<TextBlock> {
    blocks.sort_by(|a, b| a.center_y().total_cmp(&b.center_y()));
    let mut lines: Vec<Vec<TextBlock>> = Vec::new();
    for block in blocks {
        if let Some(last_line) = lines.last_mut() {
            if (block.center_y() - average_line_y(last_line)).abs() <= LINE_Y_TOLERANCE {
                last_line.push(block);
                continue;
            }
        }
        lines.push(vec![block]);
    }
    lines.sort_by(|a, b| average_line_y(a).total_cmp(&average_line_y(b)));
    let mut ordered_blocks = Vec::new();
    for mut line in lines {
        line.sort_by(|a, b| a.x_min.total_cmp(&b.x_min));
        ordered_blocks.extend(line);
    }
    ordered_blocks
}

fn group_inventory_items(blocks: Vec<TextBlock>) -> Vec<String> {
    let mut grid: BTreeMap<(usize, usize), Vec<TextBlock>> = BTreeMap::new();
    for block in blocks {
        if let (Some(r), Some(c)) = (block.row, block.col) {
            grid.entry((r, c)).or_default().push(block);
        }
    }
    let mut items = Vec::new();
    for (_, cell_blocks) in grid {
        let sorted_blocks = sort_cell_blocks(cell_blocks);
        let item_words: Vec<String> = sorted_blocks.into_iter().map(|b| b.text).collect();
        items.push(item_words.join(" "));
    }
    items
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    ort::init().with_name("WarframeOCR").commit();

    let image_path = "image.png";
    let img = image::open(image_path)?;
    let image_width = img.width() as f32;
    let image_height = img.height() as f32;
    
    // For pure ONNX, we use the original color image, not inverted! Paddle Det likes the gold colors.
    let dictionary = load_dictionary();

    println!("Loading ONNX models...");
    let mut det_session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("det_model.onnx")?;
        
    let mut rec_session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("rec_model.onnx")?;

    println!("Running DBNet Text Detector...");
    let boxes = run_detector(&mut det_session, &img)?;

    let safe_top = image_height * SAFE_TOP_RATIO;
    let safe_bottom = image_height * SAFE_BOTTOM_RATIO;
    let mut text_blocks = Vec::new();

    println!("Running CRNN Recognizer on {} detected crops...", boxes.len());
    for (x_min, y_min, x_max, y_max) in boxes {
        // Safe Zone Filtering BEFORE running recognizer to save CPU time
        if y_min < safe_top || y_max > safe_bottom { continue; }

        // Crop the original image using the coordinates
        let width = (x_max - x_min) as u32;
        let height = (y_max - y_min) as u32;
        if width == 0 || height == 0 { continue; }

        let crop = img.crop_imm(x_min as u32, y_min as u32, width, height);

        if let Some((raw_text, score)) = recognize_text(&mut rec_session, &crop, &dictionary)? {
            if score > 0.75 { // Text Score Threshold
                if let Some(cleaned) = clean_text(&raw_text) {
                    text_blocks.push(TextBlock {
                        text: cleaned,
                        score,
                        x_min, y_min, x_max, y_max,
                        row: None, col: None,
                    });
                }
            }
        }
    }

    println!("Detected {} valid text blocks. Applying Grid Binning...", text_blocks.len());
    assign_grid_positions(&mut text_blocks, image_width, image_height);
    let inventory_items = group_inventory_items(text_blocks);

    println!("\n*** Grouped Inventory Items ***");
    for item in inventory_items {
        println!("{}", item);
    }

    Ok(())
}

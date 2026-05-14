pub mod layouts;
pub mod ocr;
pub mod pipeline;
pub mod text;

use crate::{
    ocr::PaddleOcrEngine,
    pipeline::{ItemLayout, ItemPipeline, PipelineOutput},
    text::WarframeTextNormalizer,
};

pub fn run_with_layout<L>(
    pipeline: &ItemPipeline<WarframeTextNormalizer>,
    ocr: &mut PaddleOcrEngine,
    cropped_image: &image::DynamicImage,
    layout: L,
    heading: &str,
    debug_blocks: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    L: ItemLayout<Item = String>,
{
    println!("Running OCR pipeline...");
    let PipelineOutput { text_blocks, items } = pipeline.run(ocr, cropped_image, &layout)?;

    println!(
        "Detected {} valid text blocks. Applying layout grouping...",
        text_blocks.len()
    );
    if debug_blocks {
        for block in &text_blocks {
            println!(
                "[{:.0},{:.0} {:.0}x{:.0}] {:.2} {}",
                block.bounds.x_min,
                block.bounds.y_min,
                block.bounds.width(),
                block.bounds.height(),
                block.score,
                block.text
            );
        }
    }

    println!("\n*** {heading} ***");
    for item in items {
        println!("{item}");
    }

    Ok(())
}

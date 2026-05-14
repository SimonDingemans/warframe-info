use std::error::Error;

use wf_info_ocr::{
    fixture_dir, image_paths,
    layouts::RewardScreenLayout,
    load_ocr_engine,
    pipeline::{ItemPipeline, PipelineOutput},
    text::WarframeTextNormalizer,
};

fn main() -> Result<(), Box<dyn Error>> {
    let image_dir = fixture_dir("reward_screen");
    let image_paths = image_paths(&image_dir)?;
    if image_paths.is_empty() {
        return Err(format!(
            "no cropped reward screen images found in {}",
            image_dir.display()
        )
        .into());
    }

    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine()?;

    for image_path in image_paths {
        println!("=============");
        println!("source: {}", image_path.display());
        let image = image::open(&image_path)?;
        let PipelineOutput {
            text_blocks: _,
            items,
        } = pipeline.run(&mut ocr, &image, &RewardScreenLayout::default())?;

        for item in items {
            println!("{item}");
        }
    }

    Ok(())
}

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crop::{RewardScreenCrop, ScreenCrop};

fn main() -> Result<(), Box<dyn Error>> {
    let input_dir = fixture_dir("reward_screen");
    let output_dir = output_dir("reward_screen");
    let mut input_paths = image_paths(&input_dir)?;

    input_paths.sort();

    for input_path in input_paths {
        let output_path = output_dir.join(cropped_file_name(&input_path)?);
        let image = image::open(&input_path)?;
        let cropped = RewardScreenCrop::default().crop_image(&image)?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        cropped.image.save(&output_path)?;

        println!("source: {}", input_path.display());
        println!("output: {}", output_path.display());
        println!("source size: {:?}", cropped.source_size);
        println!("crop: {:?}", cropped.crop);
    }

    Ok(())
}

fn fixture_dir(dir_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/fixtures")
        .join(dir_name)
}

fn output_dir(dir_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/tmp")
        .join(dir_name)
}

fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && is_image_file(&path) {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "png"))
}

fn cropped_file_name(path: &Path) -> Result<String, Box<dyn Error>> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("image path has no UTF-8 file stem: {}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| format!("image path has no UTF-8 extension: {}", path.display()))?;

    Ok(format!("{stem}_cropped.{extension}"))
}

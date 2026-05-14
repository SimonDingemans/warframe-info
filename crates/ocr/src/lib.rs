pub mod layouts;
pub mod ocr;
pub mod pipeline;
pub mod text;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Once,
};

use crate::{ocr::PaddleOcrEngine, pipeline::PipelineResult};

static ORT_INIT: Once = Once::new();

pub fn fixture_dir(dir_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/fixtures")
        .join(dir_name)
}

pub fn image_paths(dir: &Path) -> PipelineResult<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && is_image_file(&path) {
            paths.push(path);
        }
    }

    paths.sort();
    Ok(paths)
}

pub fn load_ocr_engine() -> PipelineResult<PaddleOcrEngine> {
    ORT_INIT.call_once(|| {
        ort::init().with_name("WarframeOCR").commit();
    });

    let detector_model_path = ocr_asset_path("det_model.onnx");
    let recognizer_model_path = ocr_asset_path("rec_model.onnx");

    PaddleOcrEngine::from_files(
        path_as_str(&detector_model_path)?,
        path_as_str(&recognizer_model_path)?,
    )
}

pub fn ocr_asset_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ocr")
        .join(relative_path)
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "png"))
}

fn path_as_str(path: &Path) -> PipelineResult<&str> {
    path.to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display()).into())
}

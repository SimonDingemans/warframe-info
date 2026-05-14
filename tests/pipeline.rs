use rstest::rstest;
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::{Mutex, Once},
};
use wf_info_ocr::{
    layouts::{InventoryGridLayout, RewardScreenLayout},
    ocr::PaddleOcrEngine,
    pipeline::ItemPipeline,
    text::WarframeTextNormalizer,
};

static ORT_INIT: Once = Once::new();
static PIPELINE_TEST_LOCK: Mutex<()> = Mutex::new(());

#[rstest(
    image_path, expected_items,
    case::prime_parts(
        "image.png",
        &[
            "Acceltra Prime Receiver",
            "Ballistica Prime Receiver",
            "Boltor Prime Barrel",
            "Boltor Prime Receiver",
            "Bronco Prime Barrel",
            "Bronco Prime Receiver",
            "Cedo Prime Stock",
            "Daikyu Prime Lower Limb",
            "Daikyu Prime String",
            "Daikyu Prime Upper Limb",
            "Destreza Prime Blade",
            "Fang Prime Blade",
            "Fang Prime Handle",
            "Garuda Prime Chassis Blueprint",
            "Guandao Prime Blade",
            "Gyre Prime Systems Blueprint",
            "Hildryn Prime Systems Blueprint",
            "Kronen Prime Handle",
        ]
    ),
    case::arcanes(
        "image2.png",
        &[
            "Arcane Aegis",
            "Arcane Battery",
            "Arcane Blessing",
            "Arcane Circumvent",
            "Arcane Concentrauion",
            "Arcane Crepuscular",
            "Arcane Expertise",
            "Arcane Fury",
            "Arcane Persistence",
            "Arcane Pistoleer",
            "Cascadia Flare",
            "Eternal Logistics",
            "Fractalized Reset",
            "Melee Afflictions",
            "Melee Influence",
            "Melee Vortex",
            "Molt Augmented",
            "Molt Efficiency",
        ]
    )
)]
fn inventory_pipeline_extracts_expected_items(
    image_path: &str,
    expected_items: &[&str],
) -> Result<(), Box<dyn Error>> {
    let _guard = PIPELINE_TEST_LOCK
        .lock()
        .expect("pipeline test lock poisoned");
    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine()?;
    let layout = InventoryGridLayout::default();
    let image = image::open(fixture_path(image_path))?;

    let output = pipeline.run(&mut ocr, &image, &layout)?;
    let expected_items: Vec<String> = expected_items.iter().map(|item| item.to_string()).collect();

    assert_eq!(output.items, expected_items);

    Ok(())
}

#[rstest(
    image_path, expected_items,
    case::four_rewards(
        "RewardScreen_1.png",
        &[
            "Sarofang Prime Handle",
            "Forma Blueprint",
            "Grendel Prime Blueprin",
            "Nautilus Prime Systems",
        ]
    )
)]
fn reward_screen_pipeline_extracts_expected_items(
    image_path: &str,
    expected_items: &[&str],
) -> Result<(), Box<dyn Error>> {
    let _guard = PIPELINE_TEST_LOCK
        .lock()
        .expect("pipeline test lock poisoned");
    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine()?;
    let layout = RewardScreenLayout::default();
    let image = image::open(fixture_path(image_path))?;

    let output = pipeline.run(&mut ocr, &image, &layout)?;
    let expected_items: Vec<String> = expected_items.iter().map(|item| item.to_string()).collect();

    assert_eq!(output.items, expected_items);

    Ok(())
}

fn load_ocr_engine() -> Result<PaddleOcrEngine, Box<dyn Error>> {
    ORT_INIT.call_once(|| {
        ort::init().with_name("WarframeOCRTests").commit();
    });

    let detector_model_path = fixture_path("det_model.onnx");
    let recognizer_model_path = fixture_path("rec_model.onnx");

    PaddleOcrEngine::from_files(
        path_as_str(&detector_model_path)?,
        path_as_str(&recognizer_model_path)?,
    )
}

fn fixture_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn path_as_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| format!("Path is not valid UTF-8: {}", path.display()).into())
}

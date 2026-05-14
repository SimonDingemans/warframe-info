use rstest::rstest;
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Mutex,
};
use wf_info_ocr::{
    layouts::{InventoryGridLayout, RewardScreenLayout},
    load_ocr_engine,
    pipeline::ItemPipeline,
    text::WarframeTextNormalizer,
};

static PIPELINE_TEST_LOCK: Mutex<()> = Mutex::new(());

#[rstest(
    image_path, expected_items,
    case::prime_parts(
        "inventory/prime_parts.png",
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
        "inventory/arcanes.png",
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
    image_path,
    case::standard_resolution("inventory/screenshot_1920x1080_cropped.png"),
    case::high_resolution("inventory/screenshot_3839x2160_cropped.png")
)]
fn inventory_pipeline_keeps_stacked_arcane_name_in_reading_order(
    image_path: &str,
) -> Result<(), Box<dyn Error>> {
    let _guard = PIPELINE_TEST_LOCK
        .lock()
        .expect("pipeline test lock poisoned");
    let pipeline = ItemPipeline::new(WarframeTextNormalizer).with_min_text_score(0.75);
    let mut ocr = load_ocr_engine()?;
    let layout = InventoryGridLayout::default();
    let image = image::open(example_fixture_path(image_path))?;

    let output = pipeline.run(&mut ocr, &image, &layout)?;

    assert!(output
        .items
        .iter()
        .any(|item| item == "Arcane Concentration"));
    assert!(!output.items.iter().any(|item| item == "Concentration"));
    assert!(!output
        .items
        .iter()
        .any(|item| item == "Concentration Arcane"));

    Ok(())
}

#[rstest(
    image_path, expected_items,
    case::four_rewards(
        "reward_screen/four_rewards.png",
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

fn fixture_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative_path)
}

fn example_fixture_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/fixtures")
        .join(relative_path)
}

use std::{
    fs,
    path::{Path, PathBuf},
};

use capture::ScreenCapture;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let screenshot = capture_windows::WindowsCapture::new()
        .capture_screen()
        .await?;
    let width = screenshot.image.width();
    let height = screenshot.image.height();
    let file_name = format!("screenshot_{width}x{height}.png");
    let output_path = output_path(&file_name);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    screenshot.image.save(&output_path)?;
    println!("saved {}", output_path.display());
    Ok(())
}

fn output_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/tmp")
        .join(file_name)
}

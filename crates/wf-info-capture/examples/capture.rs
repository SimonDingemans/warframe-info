#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let screenshot = wf_info_capture::capture_screen().await?;
    screenshot.image.save("screenshot.png")?;
    println!("saved screenshot.png");
    Ok(())
}

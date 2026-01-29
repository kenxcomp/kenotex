use anyhow::{Context, Result};
use std::process::Command;
use urlencoding::encode;

pub fn create_bear_note(title: &str, text: &str, tags: Option<&[&str]>) -> Result<()> {
    let mut url = format!(
        "bear://x-callback-url/create?title={}&text={}",
        encode(title),
        encode(text)
    );

    if let Some(tags) = tags {
        let tags_str = tags.join(",");
        url.push_str(&format!("&tags={}", encode(&tags_str)));
    }

    let output = Command::new("open")
        .arg(&url)
        .output()
        .context("Failed to open Bear URL")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to open Bear: {}", stderr);
    }

    Ok(())
}

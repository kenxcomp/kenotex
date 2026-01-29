use anyhow::{Context, Result};
use std::process::Command;
use urlencoding::encode;

pub fn create_obsidian_note(name: &str, content: &str, vault: Option<&str>) -> Result<()> {
    let mut url = format!(
        "obsidian://new?name={}&content={}",
        encode(name),
        encode(content)
    );

    if let Some(vault) = vault {
        url = format!(
            "obsidian://new?vault={}&name={}&content={}",
            encode(vault),
            encode(name),
            encode(content)
        );
    }

    let output = Command::new("open")
        .arg(&url)
        .output()
        .context("Failed to open Obsidian URL")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to open Obsidian: {}", stderr);
    }

    Ok(())
}

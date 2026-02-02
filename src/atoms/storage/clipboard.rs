use anyhow::{Context, Result};
use std::process::Command;

/// Copy text to the system clipboard using pbcopy (macOS).
pub fn clipboard_copy(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn pbcopy")?;
    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(text.as_bytes())
            .context("Failed to write to pbcopy")?;
    }
    child.wait().context("pbcopy process failed")?;
    Ok(())
}

/// Paste text from the system clipboard using pbpaste (macOS).
pub fn clipboard_paste() -> Result<String> {
    let output = Command::new("pbpaste")
        .output()
        .context("Failed to run pbpaste")?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

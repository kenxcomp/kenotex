use anyhow::{Context, Result};
use std::process::Command;

pub fn create_apple_note(title: &str, body: &str, folder: Option<&str>) -> Result<()> {
    let escaped_title = escape_applescript_string(title);
    let escaped_body = escape_applescript_string(body);

    let folder_clause = if let Some(f) = folder {
        format!("folder \"{}\"", escape_applescript_string(f))
    } else {
        "default account's first folder".to_string()
    };

    let script = format!(
        r#"tell application "Notes"
    tell {}
        make new note with properties {{name:"{}", body:"{}"}}
    end tell
end tell"#,
        folder_clause, escaped_title, escaped_body
    );

    run_applescript(&script).context("Failed to create Apple Note")
}

fn escape_applescript_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn run_applescript(script: &str) -> Result<()> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("Failed to execute osascript")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("AppleScript error: {}", stderr);
    }

    Ok(())
}

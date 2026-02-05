use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::process::Command;

pub fn create_reminder(
    title: &str,
    notes: Option<&str>,
    due_date: Option<DateTime<Utc>>,
    list_name: Option<&str>,
) -> Result<()> {
    let escaped_title = escape_applescript_string(title);
    let escaped_notes = notes.map(escape_applescript_string).unwrap_or_default();

    let date_clause = if let Some(date) = due_date {
        let formatted = date.format("%B %d, %Y at %I:%M %p").to_string();
        format!(" with properties {{due date:date \"{}\"}}", formatted)
    } else {
        String::new()
    };

    let list_clause = if let Some(list) = list_name {
        format!("list \"{}\"", escape_applescript_string(list))
    } else {
        "default list".to_string()
    };

    let script = format!(
        r#"tell application "Reminders"
    tell {}
        make new reminder with properties {{name:"{}", body:"{}"}}{}
    end tell
end tell"#,
        list_clause, escaped_title, escaped_notes, date_clause
    );

    run_applescript(&script).context("Failed to create reminder")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_applescript_string() {
        assert_eq!(escape_applescript_string("test"), "test");
        assert_eq!(escape_applescript_string("test\"quote"), "test\\\"quote");
        assert_eq!(escape_applescript_string("line1\nline2"), "line1\\nline2");
    }
}

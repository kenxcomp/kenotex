use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use std::process::Command;

pub fn create_calendar_event(
    title: &str,
    notes: Option<&str>,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
    calendar_name: Option<&str>,
) -> Result<()> {
    let escaped_title = escape_applescript_string(title);
    let escaped_notes = notes.map(escape_applescript_string).unwrap_or_default();

    let start_formatted = start_date.format("%B %d, %Y at %I:%M %p").to_string();
    let end_date = end_date.unwrap_or_else(|| start_date + Duration::hours(1));
    let end_formatted = end_date.format("%B %d, %Y at %I:%M %p").to_string();

    let calendar_clause = if let Some(cal) = calendar_name {
        format!("calendar \"{}\"", escape_applescript_string(cal))
    } else {
        "first calendar whose name is not \"\"".to_string()
    };

    let script = format!(
        r#"tell application "Calendar"
    tell {}
        make new event with properties {{summary:"{}", description:"{}", start date:date "{}", end date:date "{}"}}
    end tell
end tell"#,
        calendar_clause, escaped_title, escaped_notes, start_formatted, end_formatted
    );

    run_applescript(&script).context("Failed to create calendar event")
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

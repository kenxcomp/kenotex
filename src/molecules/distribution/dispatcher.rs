use crate::atoms::applescript::{
    create_apple_note, create_bear_note, create_calendar_event, create_obsidian_note,
    create_reminder,
};
use crate::molecules::distribution::{parse_at_time_expression, parse_time_expression};
use crate::types::{BlockType, Destinations, NotesApp, SmartBlock};

#[derive(Debug)]
pub enum DispatchResult {
    Sent,
    Skipped,
    Failed(String),
}

pub fn dispatch_block(block: &SmartBlock, destinations: &Destinations) -> DispatchResult {
    // Skip blocks already wrapped in HTML comments (previously processed)
    let trimmed = block.content.trim();
    if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
        return DispatchResult::Skipped;
    }

    match block.block_type {
        BlockType::Reminder => dispatch_reminder(block, destinations),
        BlockType::Calendar => dispatch_calendar(block, destinations),
        BlockType::Note => dispatch_note(block, destinations),
    }
}

fn dispatch_reminder(block: &SmartBlock, destinations: &Destinations) -> DispatchResult {
    if destinations.reminders.app.is_empty() {
        return DispatchResult::Skipped;
    }

    let content = strip_tag(&block.content, ":::td");
    let list_name = destinations.reminders.list.as_deref();

    // Enhanced list item detection: "- ", "* ", "- [ ]", "- []"
    let list_items: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("- [ ]")
                || trimmed.starts_with("- []")
        })
        .collect();

    if !list_items.is_empty() {
        for item in &list_items {
            // Strip list prefix: "- ", "* ", "- [ ] ", "- [] "
            let title = item
                .trim()
                .trim_start_matches("- [ ]")
                .trim_start_matches("- []")
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim();

            // Parse @time from the line
            let due_date = parse_at_time_expression(title);
            let title = strip_at_time(title);

            if let Err(e) = create_reminder(&title, None, due_date, list_name) {
                return DispatchResult::Failed(format!("Reminder failed: {}", e));
            }
        }
        return DispatchResult::Sent;
    }

    // Single reminder with title/body
    let (title, body) = extract_title_body(&content);
    let body_ref = if body.is_empty() {
        None
    } else {
        Some(body.as_str())
    };

    // Parse @time from content
    let due_date = parse_at_time_expression(&content);

    let title = strip_at_time(&title);

    match create_reminder(&title, body_ref, due_date, list_name) {
        Ok(()) => DispatchResult::Sent,
        Err(e) => DispatchResult::Failed(format!("Reminder failed: {}", e)),
    }
}

fn dispatch_calendar(block: &SmartBlock, destinations: &Destinations) -> DispatchResult {
    if destinations.calendar.app.is_empty() {
        return DispatchResult::Skipped;
    }

    let content = strip_tag(&block.content, ":::cal");
    let (title, body) = extract_title_body(&content);
    let body_ref = if body.is_empty() {
        None
    } else {
        Some(body.as_str())
    };

    // Try @time first, fallback to natural language
    let start_date = match parse_at_time_expression(&content) {
        Some(dt) => dt,
        None => match parse_time_expression(&content) {
            Some(dt) => dt,
            None => return DispatchResult::Failed("Could not parse time".to_string()),
        },
    };

    let calendar_name = destinations.calendar.calendar_name.as_deref();

    let title = strip_at_time(&title);

    match create_calendar_event(&title, body_ref, start_date, None, calendar_name) {
        Ok(()) => DispatchResult::Sent,
        Err(e) => DispatchResult::Failed(format!("Calendar failed: {}", e)),
    }
}

fn dispatch_note(block: &SmartBlock, destinations: &Destinations) -> DispatchResult {
    let notes_app = match destinations.notes.app {
        Some(app) => app,
        None => return DispatchResult::Skipped,
    };

    let content = strip_tag(&block.content, ":::note");
    let (title, body) = extract_title_body(&content);

    let result = match notes_app {
        NotesApp::AppleNotes => {
            let folder = destinations.notes.folder.as_deref();
            create_apple_note(&title, &body, folder)
        }
        NotesApp::Bear => create_bear_note(&title, &body, None),
        NotesApp::Obsidian => {
            let vault = destinations.notes.vault.as_deref();
            create_obsidian_note(&title, &body, vault)
        }
    };

    match result {
        Ok(()) => DispatchResult::Sent,
        Err(e) => DispatchResult::Failed(format!("Note failed: {}", e)),
    }
}

/// Strip a tag prefix (e.g. ":::td") from the first line if present.
fn strip_tag(content: &str, tag: &str) -> String {
    let mut lines = content.lines();
    if let Some(first_line) = lines.next() {
        let stripped_first = first_line.trim_start_matches(tag).trim();
        let rest: Vec<&str> = lines.collect();
        if rest.is_empty() {
            stripped_first.to_string()
        } else {
            format!("{}\n{}", stripped_first, rest.join("\n"))
        }
    } else {
        String::new()
    }
}

/// Extract title (first line) and body (remaining lines) from content.
fn extract_title_body(content: &str) -> (String, String) {
    let mut lines = content.lines();
    let title = lines.next().unwrap_or("").trim().to_string();
    let body: String = lines.collect::<Vec<&str>>().join("\n").trim().to_string();
    (title, body)
}

/// Strip the `@time` expression from text, returning only the descriptive content.
fn strip_at_time(text: &str) -> String {
    if let Some(at_pos) = text.find('@') {
        let before = text[..at_pos].trim_end();
        let remaining = &text[at_pos + 1..];
        let mut end_pos = 0;
        for (i, ch) in remaining.char_indices() {
            if ch.is_whitespace() {
                break;
            }
            if ch.is_ascii_punctuation() && ch != ':' {
                break;
            }
            end_pos = i + ch.len_utf8();
        }
        let after = &remaining[end_pos..];
        format!("{}{}", before, after).trim().to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_tag_reminder() {
        // Content should already be tag-free from parser
        assert_eq!(strip_tag("Buy milk", ":::td"), "Buy milk");
    }

    #[test]
    fn test_strip_tag_no_tag() {
        assert_eq!(strip_tag("Buy milk", ":::td"), "Buy milk");
    }

    #[test]
    fn test_strip_tag_multiline() {
        let input = "Meeting tomorrow\nWith team";
        let result = strip_tag(input, ":::cal");
        assert_eq!(result, "Meeting tomorrow\nWith team");
    }

    #[test]
    fn test_extract_title_body() {
        let (title, body) = extract_title_body("My Title\nLine 1\nLine 2");
        assert_eq!(title, "My Title");
        assert_eq!(body, "Line 1\nLine 2");
    }

    #[test]
    fn test_extract_title_body_no_body() {
        let (title, body) = extract_title_body("My Title");
        assert_eq!(title, "My Title");
        assert_eq!(body, "");
    }

    #[test]
    fn test_dispatch_reminder_skipped_when_empty_app() {
        let block = SmartBlock::new(
            "t1".to_string(),
            "Buy milk".to_string(), // Content no longer includes tags
            BlockType::Reminder,
        );
        let mut destinations = Destinations::default();
        destinations.reminders.app = String::new();

        let result = dispatch_block(&block, &destinations);
        assert!(matches!(result, DispatchResult::Skipped));
    }

    #[test]
    fn test_dispatch_calendar_skipped_when_empty_app() {
        let block = SmartBlock::new(
            "t1".to_string(),
            "Meeting".to_string(), // Content no longer includes tags
            BlockType::Calendar,
        );
        let mut destinations = Destinations::default();
        destinations.calendar.app = String::new();

        let result = dispatch_block(&block, &destinations);
        assert!(matches!(result, DispatchResult::Skipped));
    }

    #[test]
    fn test_dispatch_note_skipped_when_none_app() {
        let block = SmartBlock::new(
            "t1".to_string(),
            "Hello".to_string(), // Content no longer includes tags
            BlockType::Note,
        );
        let mut destinations = Destinations::default();
        destinations.notes.app = None;

        let result = dispatch_block(&block, &destinations);
        assert!(matches!(result, DispatchResult::Skipped));
    }

    #[test]
    fn test_dispatch_skips_commented_block() {
        let block = SmartBlock::new(
            "t1".to_string(),
            "<!-- Buy milk -->".to_string(), // Tags removed by parser
            BlockType::Reminder,
        );
        let destinations = Destinations::default();

        let result = dispatch_block(&block, &destinations);
        assert!(matches!(result, DispatchResult::Skipped));
    }

    #[test]
    fn test_dispatch_skips_multiline_commented_block() {
        let block = SmartBlock::new(
            "t1".to_string(),
            "<!-- Meeting tomorrow\nWith team -->".to_string(), // Tags removed by parser
            BlockType::Calendar,
        );
        let destinations = Destinations::default();

        let result = dispatch_block(&block, &destinations);
        assert!(matches!(result, DispatchResult::Skipped));
    }

    // --- strip_at_time tests ---

    #[test]
    fn test_strip_at_time_simple() {
        assert_eq!(strip_at_time("buy cake @3pm"), "buy cake");
    }

    #[test]
    fn test_strip_at_time_chinese() {
        assert_eq!(strip_at_time("买蛋糕 @明天下午3点"), "买蛋糕");
    }

    #[test]
    fn test_strip_at_time_no_at() {
        assert_eq!(strip_at_time("buy cake"), "buy cake");
    }

    #[test]
    fn test_strip_at_time_middle() {
        assert_eq!(strip_at_time("buy @3pm cake"), "buy cake");
    }

    #[test]
    fn test_strip_at_time_with_colon() {
        assert_eq!(strip_at_time("meeting @3:30pm"), "meeting");
    }
}

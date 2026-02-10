use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
    #[serde(default)]
    pub selected: bool,
}

impl Note {
    pub fn new(id: String, title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            content,
            created_at: now,
            updated_at: now,
            is_archived: false,
            selected: false,
        }
    }

    pub fn extract_title(content: &str) -> String {
        let first_line = content.lines().next().unwrap_or("Untitled");
        let title = first_line
            .trim_start_matches('#')
            .trim_start_matches(' ')
            .trim();
        if title.is_empty() {
            "Untitled".to_string()
        } else {
            title.chars().take(50).collect()
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.title = Self::extract_title(&self.content);
        self.updated_at = Utc::now();
    }

    pub fn preview(&self, max_len: usize) -> String {
        let preview_content = self
            .content
            .lines()
            .find(|line| !line.trim().starts_with('#') && !line.trim().is_empty())
            .unwrap_or("");

        if preview_content.len() > max_len {
            format!("{}...", &preview_content[..max_len])
        } else {
            preview_content.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let note = Note::new(
            "id1".to_string(),
            "Title".to_string(),
            "Content".to_string(),
        );
        assert_eq!(note.id, "id1");
        assert_eq!(note.title, "Title");
        assert_eq!(note.content, "Content");
        assert!(!note.is_archived);
        assert!(!note.selected);
    }

    #[test]
    fn test_new_timestamps() {
        let before = Utc::now();
        let note = Note::new("id1".to_string(), "T".to_string(), "C".to_string());
        let after = Utc::now();
        assert!(note.created_at >= before && note.created_at <= after);
        assert_eq!(note.created_at, note.updated_at);
    }

    #[test]
    fn test_extract_title_plain() {
        assert_eq!(Note::extract_title("Hello World"), "Hello World");
    }

    #[test]
    fn test_extract_title_with_hash() {
        assert_eq!(Note::extract_title("# My Title"), "My Title");
    }

    #[test]
    fn test_extract_title_multiple_hashes() {
        assert_eq!(Note::extract_title("### Heading Three"), "Heading Three");
    }

    #[test]
    fn test_extract_title_empty() {
        assert_eq!(Note::extract_title(""), "Untitled");
    }

    #[test]
    fn test_extract_title_blank_first_line() {
        assert_eq!(Note::extract_title("   \nsecond line"), "Untitled");
    }

    #[test]
    fn test_extract_title_truncated() {
        let long_title = "A".repeat(100);
        let extracted = Note::extract_title(&long_title);
        assert_eq!(extracted.len(), 50);
    }

    #[test]
    fn test_extract_title_multiline() {
        assert_eq!(Note::extract_title("First\nSecond\nThird"), "First");
    }

    #[test]
    fn test_update_content() {
        let mut note = Note::new(
            "id1".to_string(),
            "Old".to_string(),
            "old content".to_string(),
        );
        let old_updated = note.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        note.update_content("# New Title\nnew body".to_string());
        assert_eq!(note.content, "# New Title\nnew body");
        assert_eq!(note.title, "New Title");
        assert!(note.updated_at >= old_updated);
    }

    #[test]
    fn test_preview_short() {
        let note = Note::new(
            "id1".to_string(),
            "T".to_string(),
            "# Heading\nPreview text here".to_string(),
        );
        assert_eq!(note.preview(100), "Preview text here");
    }

    #[test]
    fn test_preview_truncated() {
        let note = Note::new(
            "id1".to_string(),
            "T".to_string(),
            "This is a long preview line".to_string(),
        );
        let preview = note.preview(10);
        assert_eq!(preview, "This is a ...");
    }

    #[test]
    fn test_preview_skips_heading_and_empty() {
        let note = Note::new(
            "id1".to_string(),
            "T".to_string(),
            "# Heading\n\nActual content".to_string(),
        );
        assert_eq!(note.preview(50), "Actual content");
    }

    #[test]
    fn test_extract_title_cjk() {
        assert_eq!(Note::extract_title("# 你好世界"), "你好世界");
    }

    #[test]
    fn test_default_selected_false() {
        let note = Note::new(
            "id1".to_string(),
            "Title".to_string(),
            "Content".to_string(),
        );
        assert!(!note.selected);
        assert!(!note.is_archived);
    }
}

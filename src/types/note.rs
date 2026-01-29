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
            .skip_while(|line| line.trim().starts_with('#') || line.trim().is_empty())
            .next()
            .unwrap_or("");

        if preview_content.len() > max_len {
            format!("{}...", &preview_content[..max_len])
        } else {
            preview_content.to_string()
        }
    }
}

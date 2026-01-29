use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Reminder,
    Calendar,
    Note,
}

impl BlockType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockType::Reminder => "REMINDER",
            BlockType::Calendar => "CALENDAR",
            BlockType::Note => "NOTE",
        }
    }

    pub fn target_app(&self) -> &'static str {
        match self {
            BlockType::Reminder => "Apple Reminders",
            BlockType::Calendar => "Apple Calendar",
            BlockType::Note => "Apple Notes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartBlock {
    pub id: String,
    pub content: String,
    pub block_type: BlockType,
    pub status: ProcessingStatus,
}

impl SmartBlock {
    pub fn new(id: String, content: String, block_type: BlockType) -> Self {
        Self {
            id,
            content,
            block_type,
            status: ProcessingStatus::Pending,
        }
    }

    pub fn preview(&self, max_len: usize) -> String {
        let preview = self
            .content
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches(":::td")
            .trim_start_matches(":::cal")
            .trim_start_matches(":::note")
            .trim();

        if preview.len() > max_len {
            format!("{}...", &preview[..max_len])
        } else {
            preview.to_string()
        }
    }
}

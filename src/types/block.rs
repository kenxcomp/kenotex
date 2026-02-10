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
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartBlock {
    pub id: String,
    pub content: String,
    pub block_type: BlockType,
    pub status: ProcessingStatus,
    pub original_range: Option<(usize, usize)>,
}

impl SmartBlock {
    pub fn new(id: String, content: String, block_type: BlockType) -> Self {
        Self {
            id,
            content,
            block_type,
            status: ProcessingStatus::Pending,
            original_range: None,
        }
    }

    pub fn with_range(mut self, start: usize, end: usize) -> Self {
        self.original_range = Some((start, end));
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_as_str() {
        assert_eq!(BlockType::Reminder.as_str(), "REMINDER");
        assert_eq!(BlockType::Calendar.as_str(), "CALENDAR");
        assert_eq!(BlockType::Note.as_str(), "NOTE");
    }

    #[test]
    fn test_block_type_target_app() {
        assert_eq!(BlockType::Reminder.target_app(), "Apple Reminders");
        assert_eq!(BlockType::Calendar.target_app(), "Apple Calendar");
        assert_eq!(BlockType::Note.target_app(), "Apple Notes");
    }

    #[test]
    fn test_smart_block_new() {
        let block = SmartBlock::new(
            "b1".to_string(),
            "Buy milk".to_string(),
            BlockType::Reminder,
        );
        assert_eq!(block.id, "b1");
        assert_eq!(block.content, "Buy milk");
        assert_eq!(block.block_type, BlockType::Reminder);
        assert_eq!(block.status, ProcessingStatus::Pending);
        assert!(block.original_range.is_none());
    }

    #[test]
    fn test_smart_block_with_range() {
        let block = SmartBlock::new("b1".to_string(), "content".to_string(), BlockType::Calendar)
            .with_range(5, 10);
        assert_eq!(block.original_range, Some((5, 10)));
    }

    #[test]
    fn test_smart_block_preview_short() {
        let block = SmartBlock::new(
            "b1".to_string(),
            "Buy milk".to_string(),
            BlockType::Reminder,
        );
        assert_eq!(block.preview(50), "Buy milk");
    }

    #[test]
    fn test_smart_block_preview_truncated() {
        let block = SmartBlock::new(
            "b1".to_string(),
            "This is a very long reminder text".to_string(),
            BlockType::Reminder,
        );
        let preview = block.preview(10);
        assert_eq!(preview, "This is a ...");
    }

    #[test]
    fn test_smart_block_preview_strips_tag() {
        let block = SmartBlock::new(
            "b1".to_string(),
            ":::td Buy milk".to_string(),
            BlockType::Reminder,
        );
        assert_eq!(block.preview(50), "Buy milk");
    }

    #[test]
    fn test_smart_block_preview_strips_cal_tag() {
        let block = SmartBlock::new(
            "b1".to_string(),
            ":::cal Team meeting".to_string(),
            BlockType::Calendar,
        );
        assert_eq!(block.preview(50), "Team meeting");
    }

    #[test]
    fn test_smart_block_preview_strips_note_tag() {
        let block = SmartBlock::new(
            "b1".to_string(),
            ":::note Random thought".to_string(),
            BlockType::Note,
        );
        assert_eq!(block.preview(50), "Random thought");
    }

    #[test]
    fn test_processing_status_values() {
        let block = SmartBlock::new("b1".to_string(), "c".to_string(), BlockType::Reminder);
        assert_eq!(block.status, ProcessingStatus::Pending);
    }

    #[test]
    fn test_smart_block_with_range_chaining() {
        let block = SmartBlock::new(
            "b1".to_string(),
            "Buy milk".to_string(),
            BlockType::Reminder,
        )
        .with_range(1, 3);
        assert_eq!(block.id, "b1");
        assert_eq!(block.block_type, BlockType::Reminder);
        assert_eq!(block.original_range, Some((1, 3)));
        assert_eq!(block.status, ProcessingStatus::Pending);
    }
}

use regex::Regex;

use crate::types::{BlockType, SmartBlock};

pub fn parse_smart_blocks(text: &str) -> Vec<SmartBlock> {
    let raw_blocks: Vec<&str> = text.split("\n\n").collect();

    raw_blocks
        .into_iter()
        .enumerate()
        .filter_map(|(index, block)| {
            let trimmed = block.trim();
            if trimmed.is_empty() {
                return None;
            }

            let block_type = detect_block_type(trimmed);

            Some(SmartBlock::new(
                format!("block-{}", index),
                trimmed.to_string(),
                block_type,
            ))
        })
        .collect()
}

fn detect_block_type(content: &str) -> BlockType {
    if content.contains(":::td") {
        return BlockType::Reminder;
    }
    if content.contains(":::cal") {
        return BlockType::Calendar;
    }
    if content.contains(":::note") {
        return BlockType::Note;
    }

    if content.contains("- [ ]") || content.contains("- []") {
        return BlockType::Reminder;
    }

    let time_pattern = Regex::new(
        r"(?i)(tomorrow|today|morning|evening|monday|tuesday|wednesday|thursday|friday|saturday|sunday|daily|weekly|\d{1,2}(am|pm)| at \d)"
    ).unwrap();

    if time_pattern.is_match(content) {
        return BlockType::Calendar;
    }

    let chinese_time_pattern = Regex::new(
        r"(明天|今天|后天|下周|周一|周二|周三|周四|周五|周六|周日|上午|下午|晚上|早上)"
    ).unwrap();

    if chinese_time_pattern.is_match(content) {
        return BlockType::Calendar;
    }

    BlockType::Note
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_tags() {
        let blocks = parse_smart_blocks(":::td Buy milk\n\n:::cal Meeting at 3pm\n\n:::note Random thought");

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, BlockType::Reminder);
        assert_eq!(blocks[1].block_type, BlockType::Calendar);
        assert_eq!(blocks[2].block_type, BlockType::Note);
    }

    #[test]
    fn test_checkbox_detection() {
        let blocks = parse_smart_blocks("- [ ] Task 1\n- [ ] Task 2");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Reminder);
    }

    #[test]
    fn test_time_expression_detection() {
        let blocks = parse_smart_blocks("Meeting tomorrow at 10am");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Calendar);
    }

    #[test]
    fn test_chinese_time_detection() {
        let blocks = parse_smart_blocks("明天早上开会");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Calendar);
    }

    #[test]
    fn test_default_to_note() {
        let blocks = parse_smart_blocks("Just some random text without any markers");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Note);
    }
}

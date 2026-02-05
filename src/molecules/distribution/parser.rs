use regex::Regex;

use crate::types::{BlockType, SmartBlock};

pub fn parse_smart_blocks(text: &str) -> Vec<SmartBlock> {
    let mut blocks = Vec::new();
    let mut block_index = 0;
    let mut pos = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    while pos < len {
        // Skip leading whitespace/newlines between blocks
        while pos < len
            && (bytes[pos] == b'\n'
                || bytes[pos] == b' '
                || bytes[pos] == b'\t'
                || bytes[pos] == b'\r')
        {
            pos += 1;
        }
        if pos >= len {
            break;
        }

        let block_start = pos;

        // Find end of this block: look for double newline or end of text
        let block_end = loop {
            if pos >= len {
                break len;
            }
            if pos + 1 < len && bytes[pos] == b'\n' && bytes[pos + 1] == b'\n' {
                break pos;
            }
            pos += 1;
        };

        let block_text = &text[block_start..block_end];
        let trimmed = block_text.trim();
        if !trimmed.is_empty() {
            let block_type = detect_block_type(trimmed);
            let smart_block = SmartBlock::new(
                format!("block-{}", block_index),
                trimmed.to_string(),
                block_type,
            )
            .with_range(block_start, block_end);
            blocks.push(smart_block);
            block_index += 1;
        }

        // Skip past the double newline separator
        if pos < len {
            pos += 1; // skip first \n
            if pos < len && bytes[pos] == b'\n' {
                pos += 1; // skip second \n
            }
        }
    }

    blocks
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

    let chinese_time_pattern =
        Regex::new(r"(明天|今天|后天|下周|周一|周二|周三|周四|周五|周六|周日|上午|下午|晚上|早上)")
            .unwrap();

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
        let blocks =
            parse_smart_blocks(":::td Buy milk\n\n:::cal Meeting at 3pm\n\n:::note Random thought");

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

    #[test]
    fn test_original_range_tracking() {
        let text = ":::td Buy milk\n\n:::cal Meeting at 3pm\n\n:::note Random thought";
        let blocks = parse_smart_blocks(text);

        assert_eq!(blocks.len(), 3);

        // Verify ranges point to correct content
        for block in &blocks {
            let (start, end) = block.original_range.unwrap();
            let slice = &text[start..end];
            assert!(slice.contains(&block.content));
        }
    }

    #[test]
    fn test_range_single_block() {
        let text = "Just some text";
        let blocks = parse_smart_blocks(text);

        assert_eq!(blocks.len(), 1);
        let (start, end) = blocks[0].original_range.unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, text.len());
    }
}

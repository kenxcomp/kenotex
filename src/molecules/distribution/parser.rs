use crate::types::{BlockType, SmartBlock};

/// Parse smart blocks from text using strict tag-only system.
/// Returns (blocks, warnings) where warnings contain parsing errors.
pub fn parse_smart_blocks(text: &str) -> (Vec<SmartBlock>, Vec<String>) {
    let mut blocks = Vec::new();
    let mut warnings = Vec::new();
    let mut block_index = 0;

    // Find all opening tags with their positions
    let tag_positions = find_all_tags(text);

    for (tag_type, tag_start_pos, line_num) in tag_positions {
        // Find corresponding closing tag ":::" on its own line
        match find_closing_tag(text, tag_start_pos) {
            Some(closing_pos) => {
                // Extract content between tags (excluding tag lines)
                let content = extract_content(text, tag_start_pos, closing_pos);

                if !content.trim().is_empty() {
                    // Create SmartBlock
                    let block = SmartBlock::new(
                        format!("block-{}", block_index),
                        content,
                        tag_type_to_block_type(tag_type),
                    )
                    .with_range(tag_start_pos, closing_pos);

                    blocks.push(block);
                    block_index += 1;
                }
            }
            None => {
                // Unclosed tag: add warning and skip
                warnings.push(format!(
                    "Unclosed tag at line {}: {} not terminated with :::",
                    line_num,
                    tag_type.as_str()
                ));
            }
        }
    }

    (blocks, warnings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagType {
    Td,
    Cal,
    Note,
}

impl TagType {
    fn as_str(&self) -> &'static str {
        match self {
            TagType::Td => ":::td",
            TagType::Cal => ":::cal",
            TagType::Note => ":::note",
        }
    }
}

/// Find all opening tags with their positions and line numbers.
/// Returns Vec<(TagType, byte_pos, line_num)>
fn find_all_tags(text: &str) -> Vec<(TagType, usize, usize)> {
    let mut tags = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut line_num = 1;

    while pos < len {
        // Check if we're at line start
        let at_line_start = pos == 0 || bytes[pos - 1] == b'\n';

        if at_line_start && pos + 5 <= len {
            // Check for opening tags
            let tag_type = if &bytes[pos..pos + 5] == b":::td" {
                Some(TagType::Td)
            } else if pos + 6 <= len && &bytes[pos..pos + 6] == b":::cal" {
                Some(TagType::Cal)
            } else if pos + 7 <= len && &bytes[pos..pos + 7] == b":::note" {
                Some(TagType::Note)
            } else {
                None
            };

            if let Some(tag) = tag_type {
                let tag_len = tag.as_str().len();
                // Verify tag is followed by newline or end of text
                if pos + tag_len >= len || bytes[pos + tag_len] == b'\n' {
                    tags.push((tag, pos, line_num));
                    pos += tag_len;
                    continue;
                }
            }
        }

        // Move to next character
        if bytes[pos] == b'\n' {
            line_num += 1;
        }
        pos += 1;
    }

    tags
}

/// Find closing tag ":::" on its own line after start_pos.
/// Returns the byte position of the closing tag line end (after newline or at EOF).
fn find_closing_tag(text: &str, start_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();

    // Skip to next line after opening tag
    let mut pos = start_pos;
    while pos < len && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < len {
        pos += 1; // Skip the newline
    }

    while pos < len {
        // Check if we're at line start
        if pos == 0 || bytes[pos - 1] == b'\n' {
            // Check for ":::" at line start
            if pos + 3 <= len && &bytes[pos..pos + 3] == b":::" {
                // Verify it's on its own line (followed by newline or EOF)
                let after_closing = pos + 3;
                if after_closing >= len || bytes[after_closing] == b'\n' {
                    // Return position after the closing tag (including newline if present)
                    return Some(if after_closing < len {
                        after_closing + 1
                    } else {
                        after_closing
                    });
                }
            }
        }
        pos += 1;
    }

    None
}

/// Extract content between opening tag and closing tag.
/// start_pos points to the opening tag, end_pos points after the closing tag.
fn extract_content(text: &str, start_pos: usize, end_pos: usize) -> String {
    let bytes = text.as_bytes();

    // Find start of content (after opening tag line)
    let mut content_start = start_pos;
    while content_start < end_pos && bytes[content_start] != b'\n' {
        content_start += 1;
    }
    if content_start < end_pos {
        content_start += 1; // Skip the newline after opening tag
    }

    // Find end of content (start of closing tag line)
    // We need to find the line that starts with ":::"
    // Walk backwards from end_pos to find start of ":::" line
    let mut content_end = end_pos;

    // end_pos is after the newline following :::\n or at EOF after :::
    // If there's a newline, skip it
    if content_end > 0 && content_end <= bytes.len() && content_end > content_start {
        if content_end < bytes.len() && bytes[content_end - 1] == b'\n' {
            content_end -= 1; // Move before the final newline
        }
    }

    // Now walk back past the ":::" (3 bytes)
    if content_end >= 3 {
        content_end -= 3;
    }

    // Walk back to start of this line (the ":::" line)
    while content_end > content_start && bytes[content_end - 1] != b'\n' {
        content_end -= 1;
    }

    // Remove trailing newline before ":::" if present
    if content_end > content_start && content_end > 0 && bytes[content_end - 1] == b'\n' {
        content_end -= 1;
    }

    let content = &text[content_start..content_end];
    content.trim().to_string()
}

/// Map tag type to block type.
fn tag_type_to_block_type(tag: TagType) -> BlockType {
    match tag {
        TagType::Td => BlockType::Reminder,
        TagType::Cal => BlockType::Calendar,
        TagType::Note => BlockType::Note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_tags() {
        let input = ":::td\nBuy milk\n:::\n\n:::cal\nMeeting at 3pm\n:::\n\n:::note\nRandom thought\n:::";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert!(warnings.is_empty());
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, BlockType::Reminder);
        assert_eq!(blocks[0].content, "Buy milk");
        assert_eq!(blocks[1].block_type, BlockType::Calendar);
        assert_eq!(blocks[1].content, "Meeting at 3pm");
        assert_eq!(blocks[2].block_type, BlockType::Note);
        assert_eq!(blocks[2].content, "Random thought");
    }

    #[test]
    fn test_unclosed_tag_warning() {
        let input = ":::td\nBuy milk\n\nSome other text"; // Missing :::
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 0); // Block not created
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Unclosed tag"));
        assert!(warnings[0].contains("line 1"));
    }

    #[test]
    fn test_content_outside_tags_ignored() {
        let input = "Random text\n\n:::td\nBuy milk\n:::\n\nMore random text";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 1); // Only the tagged block
        assert_eq!(blocks[0].content, "Buy milk");
        assert!(warnings.is_empty()); // No warnings - just silently ignored
    }

    #[test]
    fn test_tag_content_on_same_line() {
        let input = ":::td\nBuy milk\n:::";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "Buy milk");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_mixed_tagged_and_untagged_content() {
        let input =
            "Untagged intro\n\n:::td\nTask 1\n:::\n\nUntagged middle\n\n:::note\nNote 1\n:::\n\nUntagged end";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].content, "Task 1");
        assert_eq!(blocks[1].content, "Note 1");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_multiline_content() {
        let input = ":::td\nLine 1\nLine 2\nLine 3\n:::";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "Line 1\nLine 2\nLine 3");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_original_range_tracking() {
        let text = ":::td\nBuy milk\n:::\n\n:::cal\nMeeting\n:::\n\n:::note\nThought\n:::";
        let (blocks, warnings) = parse_smart_blocks(text);

        assert!(warnings.is_empty());
        assert_eq!(blocks.len(), 3);

        // Verify ranges include the tags
        for block in &blocks {
            let (start, end) = block.original_range.unwrap();
            let slice = &text[start..end];
            // Slice should include opening tag through closing tag
            assert!(slice.starts_with(":::"));
        }
    }

    #[test]
    fn test_chinese_content() {
        let input = ":::td\n买牛奶\n:::\n\n:::note\n明天早上开会\n:::";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert!(warnings.is_empty());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].content, "买牛奶");
        assert_eq!(blocks[1].content, "明天早上开会");
    }

    #[test]
    fn test_empty_block_ignored() {
        let input = ":::td\n\n:::";
        let (blocks, warnings) = parse_smart_blocks(input);

        assert_eq!(blocks.len(), 0); // Empty content is ignored
        assert!(warnings.is_empty());
    }
}

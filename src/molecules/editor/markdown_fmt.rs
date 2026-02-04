// Pure functions for detecting and toggling Markdown formatting markers.

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFormat {
    Bold,          // **
    Italic,        // *
    Strikethrough, // ~~
    InlineCode,    // `
    CodeBlock,     // ```
}

impl MarkdownFormat {
    /// Returns the delimiter string for this format.
    pub fn marker(&self) -> &'static str {
        match self {
            MarkdownFormat::Bold => "**",
            MarkdownFormat::Italic => "*",
            MarkdownFormat::Strikethrough => "~~",
            MarkdownFormat::InlineCode => "`",
            MarkdownFormat::CodeBlock => "```",
        }
    }
}

/// Find the byte-index boundaries of an enclosing marker pair around `cursor_col`
/// (grapheme index). Returns `Some((start_byte, end_byte))` where `start_byte` is
/// the beginning of the opening marker and `end_byte` is the byte *after* the
/// closing marker.
///
/// For bold vs italic disambiguation: when looking for `*` (italic), positions
/// adjacent to another `*` are skipped.
pub fn find_enclosing_pair(
    line: &str,
    cursor_col: usize,
    format: MarkdownFormat,
) -> Option<(usize, usize)> {
    let marker = format.marker();
    let marker_len = marker.len(); // byte length

    // Collect grapheme boundaries: Vec<(byte_start, grapheme_str)>
    let graphemes: Vec<(usize, &str)> = line
        .grapheme_indices(true)
        .collect();

    if graphemes.is_empty() {
        return None;
    }

    // Convert cursor_col (grapheme index) to byte position
    let cursor_byte = if cursor_col < graphemes.len() {
        graphemes[cursor_col].0
    } else {
        line.len()
    };

    // Find all marker positions (byte offsets)
    let mut marker_positions: Vec<usize> = Vec::new();
    let mut search_from = 0;
    while let Some(pos) = line[search_from..].find(marker) {
        let abs_pos = search_from + pos;

        // Bold/italic disambiguation
        if format == MarkdownFormat::Italic {
            // Skip if this `*` is part of a `**` sequence
            let before = abs_pos > 0 && line.as_bytes().get(abs_pos - 1) == Some(&b'*');
            let after = line.as_bytes().get(abs_pos + 1) == Some(&b'*');
            if before || after {
                search_from = abs_pos + marker_len;
                continue;
            }
        }

        marker_positions.push(abs_pos);
        search_from = abs_pos + marker_len;
    }

    // Need at least 2 markers to form a pair
    if marker_positions.len() < 2 {
        return None;
    }

    // Try to find a pair that encloses the cursor
    // Pairs are formed by consecutive markers: (0,1), (2,3), etc.
    let mut i = 0;
    while i + 1 < marker_positions.len() {
        let open_byte = marker_positions[i];
        let close_byte = marker_positions[i + 1];
        let close_end = close_byte + marker_len;

        // Cursor is between open marker (inclusive of content start) and close marker end
        if cursor_byte >= open_byte && cursor_byte < close_end {
            // Convert byte positions to grapheme positions for the return
            let open_grapheme = line[..open_byte].graphemes(true).count();
            let close_end_grapheme = open_grapheme
                + line[open_byte..close_end].graphemes(true).count();
            return Some((open_grapheme, close_end_grapheme));
        }
        i += 2;
    }

    None
}

/// Toggle inline formatting on a single line at the cursor position (Normal mode).
///
/// - If cursor is inside a formatted pair, remove the markers.
/// - Otherwise, insert empty markers at cursor position.
///
/// Returns `(new_line, new_cursor_col)` where cursor_col is a grapheme index.
pub fn toggle_inline_format(
    line: &str,
    cursor_col: usize,
    format: MarkdownFormat,
) -> (String, usize) {
    let marker = format.marker();
    let marker_grapheme_len = marker.graphemes(true).count();

    if let Some((open_g, close_g)) = find_enclosing_pair(line, cursor_col, format) {
        // Remove markers
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut result = String::new();
        for (i, g) in graphemes.iter().enumerate() {
            // Skip the opening marker graphemes
            if i >= open_g && i < open_g + marker_grapheme_len {
                continue;
            }
            // Skip the closing marker graphemes
            if i >= close_g - marker_grapheme_len && i < close_g {
                continue;
            }
            result.push_str(g);
        }
        let new_col = if cursor_col >= open_g + marker_grapheme_len {
            cursor_col - marker_grapheme_len
        } else {
            open_g
        };
        (result, new_col)
    } else {
        // Insert empty markers at cursor
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let insert_pos = cursor_col.min(graphemes.len());
        let mut result = String::new();
        for g in &graphemes[..insert_pos] {
            result.push_str(g);
        }
        result.push_str(marker);
        result.push_str(marker);
        for g in &graphemes[insert_pos..] {
            result.push_str(g);
        }
        let new_col = insert_pos + marker_grapheme_len;
        (result, new_col)
    }
}

/// Toggle inline formatting on a visual selection within a single line.
///
/// - `sel_start` and `sel_end` are grapheme indices (inclusive start, exclusive end).
/// - If the selected text is already wrapped in the markers, remove them.
/// - Otherwise, wrap the selection with markers.
///
/// Returns `(new_line, new_sel_start, new_sel_end)`.
pub fn toggle_inline_format_visual(
    line: &str,
    sel_start: usize,
    sel_end: usize,
    format: MarkdownFormat,
) -> (String, usize, usize) {
    let marker = format.marker();
    let marker_grapheme_len = marker.graphemes(true).count();
    let graphemes: Vec<&str> = line.graphemes(true).collect();
    let sel_end = sel_end.min(graphemes.len());
    let sel_start = sel_start.min(sel_end);

    // Check if the selection is already wrapped
    let selected: String = graphemes[sel_start..sel_end].iter().copied().collect();
    if selected.starts_with(marker) && selected.ends_with(marker)
        && selected.len() >= marker.len() * 2
    {
        // Unwrap: remove markers from both ends
        let inner_start = sel_start + marker_grapheme_len;
        let inner_end = sel_end - marker_grapheme_len;
        let mut result = String::new();
        for g in &graphemes[..sel_start] {
            result.push_str(g);
        }
        for g in &graphemes[inner_start..inner_end] {
            result.push_str(g);
        }
        for g in &graphemes[sel_end..] {
            result.push_str(g);
        }
        (result, sel_start, sel_start + (inner_end - inner_start))
    } else {
        // Wrap: add markers around selection
        let mut result = String::new();
        for g in &graphemes[..sel_start] {
            result.push_str(g);
        }
        result.push_str(marker);
        for g in &graphemes[sel_start..sel_end] {
            result.push_str(g);
        }
        result.push_str(marker);
        for g in &graphemes[sel_end..] {
            result.push_str(g);
        }
        (
            result,
            sel_start,
            sel_end + marker_grapheme_len * 2,
        )
    }
}

/// Check if the cursor is inside a code block (``` fences).
/// Returns `Some((fence_start_row, fence_end_row))` if found.
pub fn is_inside_code_block(lines: &[String], cursor_row: usize) -> Option<(usize, usize)> {
    // Scan upward for an opening fence
    let mut open_row = None;
    for row in (0..cursor_row).rev() {
        let trimmed = lines[row].trim();
        if trimmed.starts_with("```") {
            // This could be opening or closing. We need to determine context.
            // Count fences from the start to determine if it's opening or closing.
            open_row = Some(row);
            break;
        }
    }

    let open_row = open_row?;

    // Verify it's an opening fence by counting from the start
    let fence_count = lines[..=open_row]
        .iter()
        .filter(|l| l.trim().starts_with("```"))
        .count();
    // Odd count means the fence at open_row is an opening fence
    if fence_count % 2 == 0 {
        return None;
    }

    // Scan downward for a closing fence
    for (i, line) in lines.iter().enumerate().skip(cursor_row + 1) {
        if line.trim().starts_with("```") {
            return Some((open_row, i));
        }
    }

    None
}

/// Toggle code block fences around the current line (Normal mode).
///
/// - If cursor is inside a code block, remove the fences.
/// - Otherwise, insert ``` fences around the current line, with cursor on the
///   opening fence line (after ```) for language typing.
///
/// Returns `(new_lines, new_cursor_row, new_cursor_col)`.
pub fn toggle_code_block(
    lines: &[String],
    cursor_row: usize,
) -> (Vec<String>, usize, usize) {
    if let Some((open_row, close_row)) = is_inside_code_block(lines, cursor_row) {
        // Remove fences
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() - 2);
        for (i, line) in lines.iter().enumerate() {
            if i == open_row || i == close_row {
                continue;
            }
            new_lines.push(line.clone());
        }
        let new_row = if cursor_row > open_row {
            cursor_row - 1
        } else {
            cursor_row
        };
        let new_row = new_row.min(new_lines.len().saturating_sub(1));
        (new_lines, new_row, 0)
    } else {
        // Insert fences
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() + 2);
        for (i, line) in lines.iter().enumerate() {
            if i == cursor_row {
                new_lines.push("```".to_string());
                new_lines.push(line.clone());
                new_lines.push("```".to_string());
            } else {
                new_lines.push(line.clone());
            }
        }
        // Cursor on the opening fence line, after "```" for language typing
        let new_row = cursor_row;
        let new_col = 3; // after "```"
        (new_lines, new_row, new_col)
    }
}

/// Toggle code block fences around a visual selection (Visual mode).
///
/// - If the selection is already wrapped in fences, remove them.
/// - Otherwise, wrap the selected lines with fences.
///
/// Returns `(new_lines, new_cursor_row, new_cursor_col)`.
pub fn toggle_code_block_visual(
    lines: &[String],
    start_row: usize,
    end_row: usize,
) -> (Vec<String>, usize, usize) {
    let start_row = start_row.min(lines.len().saturating_sub(1));
    let end_row = end_row.min(lines.len().saturating_sub(1));

    // Check if already wrapped in fences
    let start_trimmed = lines[start_row].trim();
    let end_trimmed = lines[end_row].trim();
    let is_wrapped = start_trimmed.starts_with("```") && end_trimmed == "```";

    if is_wrapped {
        // Remove fences
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() - 2);
        for (i, line) in lines.iter().enumerate() {
            if i == start_row || i == end_row {
                continue;
            }
            new_lines.push(line.clone());
        }
        (new_lines, start_row, 0)
    } else {
        // Wrap with fences
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() + 2);
        for (i, line) in lines.iter().enumerate() {
            if i == start_row {
                new_lines.push("```".to_string());
            }
            new_lines.push(line.clone());
            if i == end_row {
                new_lines.push("```".to_string());
            }
        }
        (new_lines, start_row, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MarkdownFormat::marker ─────────────────────────────────────

    #[test]
    fn test_markers() {
        assert_eq!(MarkdownFormat::Bold.marker(), "**");
        assert_eq!(MarkdownFormat::Italic.marker(), "*");
        assert_eq!(MarkdownFormat::Strikethrough.marker(), "~~");
        assert_eq!(MarkdownFormat::InlineCode.marker(), "`");
        assert_eq!(MarkdownFormat::CodeBlock.marker(), "```");
    }

    // ── find_enclosing_pair ────────────────────────────────────────

    #[test]
    fn test_find_pair_bold_inside() {
        // "hello **world** foo"
        //  0     5 67    12 1314
        let line = "hello **world** foo";
        let result = find_enclosing_pair(line, 8, MarkdownFormat::Bold);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert_eq!(start, 6);  // opening **
        assert_eq!(end, 15);   // after closing **
    }

    #[test]
    fn test_find_pair_bold_outside() {
        let line = "hello **world** foo";
        let result = find_enclosing_pair(line, 3, MarkdownFormat::Bold);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_pair_italic_disambiguates_bold() {
        // "hello *italic* **bold** end"
        let line = "hello *italic* **bold** end";
        // Searching for italic at position inside *italic*
        let result = find_enclosing_pair(line, 7, MarkdownFormat::Italic);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_pair_inline_code() {
        let line = "hello `code` world";
        let result = find_enclosing_pair(line, 7, MarkdownFormat::InlineCode);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_pair_strikethrough() {
        let line = "hello ~~struck~~ world";
        let result = find_enclosing_pair(line, 8, MarkdownFormat::Strikethrough);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_pair_no_markers() {
        let line = "hello world";
        let result = find_enclosing_pair(line, 3, MarkdownFormat::Bold);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_pair_empty_line() {
        let result = find_enclosing_pair("", 0, MarkdownFormat::Bold);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_pair_cursor_on_opening_marker() {
        let line = "**bold** end";
        let result = find_enclosing_pair(line, 0, MarkdownFormat::Bold);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 8);
    }

    // ── toggle_inline_format (Normal mode) ─────────────────────────

    #[test]
    fn test_toggle_insert_bold_empty() {
        // Cursor at position 5 in "hello world" -> "hello****world" is wrong
        // Actually: "hello **|** world" (insert empty markers)
        let (result, col) = toggle_inline_format("hello world", 5, MarkdownFormat::Bold);
        assert_eq!(result, "hello**** world");
        assert_eq!(col, 7); // cursor between the **
    }

    #[test]
    fn test_toggle_remove_bold() {
        let (result, col) = toggle_inline_format("hello **world** foo", 8, MarkdownFormat::Bold);
        assert_eq!(result, "hello world foo");
        assert_eq!(col, 6); // cursor adjusted left
    }

    #[test]
    fn test_toggle_insert_italic() {
        let (result, col) = toggle_inline_format("hello", 5, MarkdownFormat::Italic);
        assert_eq!(result, "hello**");
        assert_eq!(col, 6);
    }

    #[test]
    fn test_toggle_remove_inline_code() {
        let (result, col) = toggle_inline_format("say `hello` there", 6, MarkdownFormat::InlineCode);
        assert_eq!(result, "say hello there");
        assert_eq!(col, 5);
    }

    #[test]
    fn test_toggle_insert_at_start() {
        let (result, col) = toggle_inline_format("hello", 0, MarkdownFormat::Bold);
        assert_eq!(result, "****hello");
        assert_eq!(col, 2);
    }

    #[test]
    fn test_toggle_empty_line() {
        let (result, col) = toggle_inline_format("", 0, MarkdownFormat::Bold);
        assert_eq!(result, "****");
        assert_eq!(col, 2);
    }

    #[test]
    fn test_toggle_line_with_only_markers() {
        // Cursor inside existing ****: should remove
        let (result, col) = toggle_inline_format("****", 2, MarkdownFormat::Bold);
        assert_eq!(result, "");
        assert_eq!(col, 0);
    }

    // ── toggle_inline_format_visual ────────────────────────────────

    #[test]
    fn test_visual_wrap_bold() {
        let (result, start, end) =
            toggle_inline_format_visual("hello world foo", 6, 11, MarkdownFormat::Bold);
        assert_eq!(result, "hello **world** foo");
        assert_eq!(start, 6);
        assert_eq!(end, 15);
    }

    #[test]
    fn test_visual_unwrap_bold() {
        let (result, start, end) =
            toggle_inline_format_visual("hello **world** foo", 6, 15, MarkdownFormat::Bold);
        assert_eq!(result, "hello world foo");
        assert_eq!(start, 6);
        assert_eq!(end, 11);
    }

    #[test]
    fn test_visual_wrap_inline_code() {
        let (result, start, end) =
            toggle_inline_format_visual("hello world", 0, 5, MarkdownFormat::InlineCode);
        assert_eq!(result, "`hello` world");
        assert_eq!(start, 0);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_visual_wrap_strikethrough() {
        let (result, _, _) =
            toggle_inline_format_visual("hello world", 6, 11, MarkdownFormat::Strikethrough);
        assert_eq!(result, "hello ~~world~~");
    }

    #[test]
    fn test_visual_empty_selection() {
        let (result, start, end) =
            toggle_inline_format_visual("hello", 3, 3, MarkdownFormat::Bold);
        // Empty selection: wraps nothing, just inserts ****
        assert_eq!(result, "hel****lo");
        assert_eq!(start, 3);
        assert_eq!(end, 7);
    }

    // ── is_inside_code_block ──────────────────────────────────────

    #[test]
    fn test_inside_code_block() {
        let lines: Vec<String> = vec![
            "hello".to_string(),
            "```".to_string(),
            "code here".to_string(),
            "```".to_string(),
            "world".to_string(),
        ];
        let result = is_inside_code_block(&lines, 2);
        assert_eq!(result, Some((1, 3)));
    }

    #[test]
    fn test_not_inside_code_block() {
        let lines: Vec<String> = vec![
            "hello".to_string(),
            "world".to_string(),
        ];
        let result = is_inside_code_block(&lines, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_outside_code_block_after_close() {
        let lines: Vec<String> = vec![
            "```".to_string(),
            "code".to_string(),
            "```".to_string(),
            "outside".to_string(),
        ];
        let result = is_inside_code_block(&lines, 3);
        assert!(result.is_none());
    }

    // ── toggle_code_block (Normal mode) ───────────────────────────

    #[test]
    fn test_toggle_code_block_insert() {
        let lines: Vec<String> = vec![
            "hello".to_string(),
            "code line".to_string(),
            "world".to_string(),
        ];
        let (new_lines, row, col) = toggle_code_block(&lines, 1);
        assert_eq!(new_lines.len(), 5);
        assert_eq!(new_lines[0], "hello");
        assert_eq!(new_lines[1], "```");
        assert_eq!(new_lines[2], "code line");
        assert_eq!(new_lines[3], "```");
        assert_eq!(new_lines[4], "world");
        assert_eq!(row, 1);
        assert_eq!(col, 3);
    }

    #[test]
    fn test_toggle_code_block_remove() {
        let lines: Vec<String> = vec![
            "hello".to_string(),
            "```rust".to_string(),
            "code".to_string(),
            "```".to_string(),
            "world".to_string(),
        ];
        let (new_lines, row, col) = toggle_code_block(&lines, 2);
        assert_eq!(new_lines.len(), 3);
        assert_eq!(new_lines[0], "hello");
        assert_eq!(new_lines[1], "code");
        assert_eq!(new_lines[2], "world");
        assert_eq!(row, 1);
    }

    // ── toggle_code_block_visual ──────────────────────────────────

    #[test]
    fn test_visual_code_block_wrap() {
        let lines: Vec<String> = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];
        let (new_lines, row, col) = toggle_code_block_visual(&lines, 0, 1);
        assert_eq!(new_lines.len(), 5);
        assert_eq!(new_lines[0], "```");
        assert_eq!(new_lines[1], "line1");
        assert_eq!(new_lines[2], "line2");
        assert_eq!(new_lines[3], "```");
        assert_eq!(new_lines[4], "line3");
        assert_eq!(row, 0);
        assert_eq!(col, 3);
    }

    #[test]
    fn test_visual_code_block_unwrap() {
        let lines: Vec<String> = vec![
            "```rust".to_string(),
            "code1".to_string(),
            "code2".to_string(),
            "```".to_string(),
            "after".to_string(),
        ];
        let (new_lines, row, col) = toggle_code_block_visual(&lines, 0, 3);
        assert_eq!(new_lines.len(), 3);
        assert_eq!(new_lines[0], "code1");
        assert_eq!(new_lines[1], "code2");
        assert_eq!(new_lines[2], "after");
    }
}

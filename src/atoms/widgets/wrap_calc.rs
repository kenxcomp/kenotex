use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Visual position of a cursor after accounting for soft-wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualPosition {
    /// Total display rows consumed by all logical lines *before* the cursor's line.
    pub rows_before: u16,
    /// Which wrapped sub-row within the cursor's logical line (0-based).
    pub wrap_row: u16,
    /// Column on the current display row (display-width units, 0-based).
    pub col: u16,
    /// Total display rows the cursor's logical line occupies.
    pub line_display_rows: u16,
}

/// Count how many display rows a single logical line occupies when wrapped at `width`.
///
/// Matches ratatui `Wrap { trim: false }`: character-level wrapping where a grapheme
/// that would exceed `width` starts a new display row.
pub fn display_rows_for_line(line: &str, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }
    let w = width as usize;
    let mut rows: u16 = 1;
    let mut col: usize = 0;

    for g in line.graphemes(true) {
        let gw = g.width();
        if gw == 0 {
            continue;
        }
        if col + gw > w {
            rows += 1;
            col = gw;
        } else {
            col += gw;
        }
    }
    rows
}

/// Compute the visual cursor position accounting for soft-wrap of all lines.
///
/// `cursor_col` is a grapheme index (not display-width).
pub fn visual_cursor_position(
    lines: &[String],
    cursor_row: usize,
    cursor_col: usize,
    width: u16,
) -> VisualPosition {
    let w = if width == 0 { 1 } else { width as usize };

    // Sum display rows for all lines before cursor_row
    let rows_before: u16 = lines
        .iter()
        .take(cursor_row)
        .map(|l| display_rows_for_line(l, width))
        .sum();

    // Compute wrap_row and col within the cursor's line
    let line = lines.get(cursor_row).map(|s| s.as_str()).unwrap_or("");
    let line_display_rows = display_rows_for_line(line, width);

    let mut wrap_row: u16 = 0;
    let mut col: usize = 0;

    for (grapheme_idx, g) in line.graphemes(true).enumerate() {
        let gw = g.width();
        if gw > 0 && col + gw > w {
            wrap_row += 1;
            col = 0;
        }
        if grapheme_idx >= cursor_col {
            break;
        }
        if gw > 0 {
            col += gw;
        }
    }

    VisualPosition {
        rows_before,
        wrap_row,
        col: col as u16,
        line_display_rows,
    }
}

/// Compute `(wrap_row, col)` for each grapheme index in `[col_start, col_end)`.
///
/// Returns one entry per grapheme in the range. Each entry gives the display-row
/// offset within this logical line and the column on that display row.
pub fn visual_positions_in_range(
    line: &str,
    col_start: usize,
    col_end: usize,
    width: u16,
) -> Vec<(u16, u16, u16)> {
    let w = if width == 0 { 1 } else { width as usize };
    let mut result = Vec::new();
    let mut wrap_row: u16 = 0;
    let mut col: usize = 0;
    let mut grapheme_idx: usize = 0;

    for g in line.graphemes(true) {
        let gw = g.width();
        if gw > 0 && col + gw > w {
            wrap_row += 1;
            col = 0;
        }
        if grapheme_idx >= col_start && grapheme_idx < col_end {
            result.push((wrap_row, col as u16, gw as u16));
        }
        if gw > 0 {
            col += gw;
        }
        grapheme_idx += 1;
    }

    // If col_end extends past line length (trailing cursor / visual selection),
    // append entries for the virtual trailing position.
    while grapheme_idx < col_end && grapheme_idx >= col_start {
        if col + 1 > w {
            wrap_row += 1;
            col = 0;
        }
        result.push((wrap_row, col as u16, 1));
        col += 1;
        grapheme_idx += 1;
    }

    result
}

/// Compute visual positions for a "virtual" block range that may extend beyond the line's content.
///
/// This is used for Visual Block mode when the selection rectangle extends past the end of shorter lines.
/// Returns positions for virtual spaces that should be rendered as part of the block selection.
///
/// # Arguments
/// * `line` - The line content
/// * `start_col` - Starting grapheme index (may be beyond line length)
/// * `end_col` - Ending grapheme index (may be beyond line length)
/// * `width` - Display width for wrapping
///
/// # Returns
/// Vector of `(wrap_row, col, width)` tuples for each position to render
pub fn virtual_block_positions(
    line: &str,
    start_col: usize,
    end_col: usize,
    width: u16,
) -> Vec<(u16, u16, u16)> {
    let w = if width == 0 { 1 } else { width as usize };
    let mut result = Vec::new();

    let graphemes: Vec<&str> = line.graphemes(true).collect();
    let _line_len = graphemes.len();

    // Start by processing actual graphemes up to line_len, tracking wrap position
    let mut wrap_row: u16 = 0;
    let mut col: usize = 0;
    let mut grapheme_idx: usize = 0;

    // Process actual graphemes to establish display position
    for g in &graphemes {
        let gw = g.width();
        if gw > 0 && col + gw > w {
            wrap_row += 1;
            col = 0;
        }
        // If this grapheme is in our range, output it
        if grapheme_idx >= start_col && grapheme_idx < end_col {
            result.push((wrap_row, col as u16, gw as u16));
        }
        if gw > 0 {
            col += gw;
        }
        grapheme_idx += 1;
    }

    // Now handle virtual positions beyond the line end
    // grapheme_idx is now at line_len, col is at the display position after last char
    while grapheme_idx < end_col {
        // Check if we need to wrap
        if col + 1 > w {
            wrap_row += 1;
            col = 0;
        }
        // Only output if in range
        if grapheme_idx >= start_col {
            result.push((wrap_row, col as u16, 1));
        }
        col += 1;
        grapheme_idx += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_rows_empty_line() {
        assert_eq!(display_rows_for_line("", 10), 1);
    }

    #[test]
    fn test_display_rows_short_line() {
        assert_eq!(display_rows_for_line("hello", 10), 1);
    }

    #[test]
    fn test_display_rows_exact_width() {
        assert_eq!(display_rows_for_line("abcde", 5), 1);
    }

    #[test]
    fn test_display_rows_wraps_once() {
        assert_eq!(display_rows_for_line("abcdef", 5), 2);
    }

    #[test]
    fn test_display_rows_wraps_twice() {
        // 11 chars, width 5 -> rows: "abcde" "fghij" "k" = 3
        assert_eq!(display_rows_for_line("abcdefghijk", 5), 3);
    }

    #[test]
    fn test_display_rows_wide_chars() {
        // Each CJK char is 2 display-width. Width 5 fits 2 chars (4 cols).
        // Third char starts at col 4, 4+2=6 > 5 -> wraps.
        // "你好" = 4 cols (row 1), "世界" = 4 cols (row 2)
        assert_eq!(display_rows_for_line("你好世界", 5), 2);
    }

    #[test]
    fn test_display_rows_wide_char_at_boundary() {
        // Width 3, "a你" -> 'a' col=1, '你' width=2, 1+2=3 <= 3, fits row 1
        assert_eq!(display_rows_for_line("a你", 3), 1);
        // Width 3, "ab你" -> 'a' col=1, 'b' col=2, '你' width=2, 2+2=4 > 3 -> wraps
        assert_eq!(display_rows_for_line("ab你", 3), 2);
    }

    #[test]
    fn test_display_rows_zero_width() {
        assert_eq!(display_rows_for_line("hello", 0), 1);
    }

    #[test]
    fn test_cursor_position_simple() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let vpos = visual_cursor_position(&lines, 0, 3, 10);
        assert_eq!(vpos.rows_before, 0);
        assert_eq!(vpos.wrap_row, 0);
        assert_eq!(vpos.col, 3);
    }

    #[test]
    fn test_cursor_position_second_line() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let vpos = visual_cursor_position(&lines, 1, 2, 10);
        assert_eq!(vpos.rows_before, 1);
        assert_eq!(vpos.wrap_row, 0);
        assert_eq!(vpos.col, 2);
    }

    #[test]
    fn test_cursor_position_wrapped_line() {
        // "abcdefgh" with width 5: "abcde" (row 0) "fgh" (row 1)
        let lines = vec!["abcdefgh".to_string()];
        // Cursor at grapheme index 6 = 'g', which is on wrap row 1, col 1
        let vpos = visual_cursor_position(&lines, 0, 6, 5);
        assert_eq!(vpos.rows_before, 0);
        assert_eq!(vpos.wrap_row, 1);
        assert_eq!(vpos.col, 1);
    }

    #[test]
    fn test_cursor_position_at_wrap_boundary() {
        // "abcde" with width 5: exactly fits row 0
        // Cursor at index 5 (end of line, Insert mode) -> wrap row 1, col 0
        let lines = vec!["abcde".to_string()];
        let vpos = visual_cursor_position(&lines, 0, 5, 5);
        assert_eq!(vpos.wrap_row, 0);
        assert_eq!(vpos.col, 5);
    }

    #[test]
    fn test_cursor_position_wrapped_affects_rows_before() {
        // Line 0: "abcdefgh" width 5 -> 2 display rows
        // Line 1: "xy"
        let lines = vec!["abcdefgh".to_string(), "xy".to_string()];
        let vpos = visual_cursor_position(&lines, 1, 1, 5);
        assert_eq!(vpos.rows_before, 2);
        assert_eq!(vpos.wrap_row, 0);
        assert_eq!(vpos.col, 1);
    }

    #[test]
    fn test_cursor_position_wide_chars() {
        // "你好世" with width 5: "你好" = 4 cols (row 0), "世" = 2 cols (row 1)
        let lines = vec!["你好世".to_string()];
        // Cursor at grapheme 2 = '世', col in row 0 was 4, 4+2=6 > 5 -> wraps
        let vpos = visual_cursor_position(&lines, 0, 2, 5);
        assert_eq!(vpos.wrap_row, 1);
        assert_eq!(vpos.col, 0);
    }

    #[test]
    fn test_cursor_position_empty_line() {
        let lines = vec!["".to_string()];
        let vpos = visual_cursor_position(&lines, 0, 0, 10);
        assert_eq!(vpos.rows_before, 0);
        assert_eq!(vpos.wrap_row, 0);
        assert_eq!(vpos.col, 0);
        assert_eq!(vpos.line_display_rows, 1);
    }

    #[test]
    fn test_visual_positions_in_range_simple() {
        let positions = visual_positions_in_range("hello", 1, 4, 10);
        // graphemes 1,2,3 -> 'e','l','l' all on row 0
        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], (0, 1, 1)); // 'e' at row 0, col 1, width 1
        assert_eq!(positions[1], (0, 2, 1)); // 'l' at row 0, col 2
        assert_eq!(positions[2], (0, 3, 1)); // 'l' at row 0, col 3
    }

    #[test]
    fn test_visual_positions_in_range_wrapped() {
        // "abcdefgh" with width 5: "abcde" (row 0) "fgh" (row 1)
        let positions = visual_positions_in_range("abcdefgh", 3, 7, 5);
        // grapheme 3 = 'd' -> row 0, col 3
        // grapheme 4 = 'e' -> row 0, col 4
        // grapheme 5 = 'f' -> row 1, col 0
        // grapheme 6 = 'g' -> row 1, col 1
        assert_eq!(positions.len(), 4);
        assert_eq!(positions[0], (0, 3, 1));
        assert_eq!(positions[1], (0, 4, 1));
        assert_eq!(positions[2], (1, 0, 1));
        assert_eq!(positions[3], (1, 1, 1));
    }

    #[test]
    fn test_visual_positions_trailing() {
        // Selection past end of line (trailing space for visual selection)
        let positions = visual_positions_in_range("ab", 0, 3, 10);
        assert_eq!(positions.len(), 3);
        assert_eq!(positions[2], (0, 2, 1)); // virtual trailing position
    }

    #[test]
    fn test_virtual_block_positions_beyond_line() {
        // Line "ab" (2 graphemes), block selection from col 3 to 5 (all virtual)
        let positions = virtual_block_positions("ab", 3, 5, 10);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], (0, 3, 1)); // Virtual space at col 3
        assert_eq!(positions[1], (0, 4, 1)); // Virtual space at col 4
    }

    #[test]
    fn test_virtual_block_positions_partial_virtual() {
        // Line "abc" (3 graphemes), block from col 2 to 5 (mix of real and virtual)
        let positions = virtual_block_positions("abc", 2, 5, 10);
        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], (0, 2, 1)); // Real 'c'
        assert_eq!(positions[1], (0, 3, 1)); // Virtual
        assert_eq!(positions[2], (0, 4, 1)); // Virtual
    }

    #[test]
    fn test_virtual_block_positions_with_cjk() {
        // Line "你好" (2 graphemes, 4 display width)
        // Grapheme 0 "你" at display cols [0,1], grapheme 1 "好" at display cols [2,3]
        // After processing both, we're at display col 4
        // Requesting graphemes 3-5 means virtual graphemes 3 and 4
        // Since grapheme 2 would be at col 4, grapheme 3 is at col 5, grapheme 4 at col 6
        let positions = virtual_block_positions("你好", 3, 5, 10);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], (0, 5, 1)); // Grapheme 3 at display col 5
        assert_eq!(positions[1], (0, 6, 1)); // Grapheme 4 at display col 6
    }

    #[test]
    fn test_virtual_block_positions_wrapping() {
        // Line "ab", width 5, block from col 3 to 8 should wrap
        let positions = virtual_block_positions("ab", 3, 8, 5);
        assert_eq!(positions.len(), 5);
        assert_eq!(positions[0], (0, 3, 1)); // col 3
        assert_eq!(positions[1], (0, 4, 1)); // col 4
        assert_eq!(positions[2], (1, 0, 1)); // wraps to row 1
        assert_eq!(positions[3], (1, 1, 1));
        assert_eq!(positions[4], (1, 2, 1));
    }
}

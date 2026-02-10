use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use super::TextBuffer;

/// Visual mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualType {
    Character,
    Line,
    Block,
}

/// Visual mode state
#[derive(Debug, Clone)]
pub struct VisualMode {
    visual_type: VisualType,
    anchor: (usize, usize), // (row, col) where selection started
}

/// Render selection data for the editor widget
#[derive(Debug, Clone)]
pub enum RenderSelection {
    CharacterRange {
        start: (usize, usize),
        end: (usize, usize),
    },
    LineRange {
        start_row: usize,
        end_row: usize,
    },
    BlockRegion {
        top_row: usize,
        bottom_row: usize,
        left_col: usize,  // display column (not grapheme index)
        right_col: usize, // display column, inclusive (not grapheme index)
    },
}

impl VisualMode {
    pub fn new(visual_type: VisualType, anchor: (usize, usize)) -> Self {
        Self {
            visual_type,
            anchor,
        }
    }

    pub fn set_type(&mut self, new_type: VisualType) {
        self.visual_type = new_type;
    }

    pub fn anchor(&self) -> (usize, usize) {
        self.anchor
    }

    pub fn render_data(&self, buffer: &TextBuffer, cursor: (usize, usize)) -> RenderSelection {
        match self.visual_type {
            VisualType::Character => {
                let (start, end) = Self::normalize_range(self.anchor, cursor);
                RenderSelection::CharacterRange { start, end }
            }
            VisualType::Line => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                RenderSelection::LineRange { start_row, end_row }
            }
            VisualType::Block => {
                let top_row = self.anchor.0.min(cursor.0);
                let bottom_row = self.anchor.0.max(cursor.0);
                let (left_col, right_col) = Self::block_display_range(buffer, self.anchor, cursor);
                RenderSelection::BlockRegion {
                    top_row,
                    bottom_row,
                    left_col,
                    right_col,
                }
            }
        }
    }

    pub fn delete_selection(&self, buffer: &mut TextBuffer, cursor: (usize, usize)) -> String {
        match self.visual_type {
            VisualType::Character => {
                let (start, end) = Self::normalize_range(self.anchor, cursor);
                buffer.delete_range(start.0, start.1, end.0, end.1 + 1)
            }
            VisualType::Line => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                let mut deleted = String::new();
                for _ in start_row..=end_row {
                    buffer.set_cursor(start_row, 0);
                    deleted.push_str(&buffer.delete_line_and_return());
                }
                deleted
            }
            VisualType::Block => {
                let top_row = self.anchor.0.min(cursor.0);
                let bottom_row = self.anchor.0.max(cursor.0);
                let (left_display, right_display) =
                    Self::block_display_range(buffer, self.anchor, cursor);

                let mut deleted = String::new();
                for row in (top_row..=bottom_row).rev() {
                    let (g_start, g_end) =
                        buffer.grapheme_range_for_display_cols(row, left_display, right_display);
                    let text = buffer.delete_range(row, g_start, row, g_end);
                    if row > top_row {
                        deleted.insert_str(0, &format!("\n{}", text));
                    } else {
                        deleted.insert_str(0, &text);
                    }
                }
                let cursor_col = buffer.grapheme_at_display_col(top_row, left_display);
                buffer.set_cursor(top_row, cursor_col);
                deleted
            }
        }
    }

    pub fn yank_selection(&self, buffer: &TextBuffer, cursor: (usize, usize)) -> String {
        match self.visual_type {
            VisualType::Character => {
                let (start, end) = Self::normalize_range(self.anchor, cursor);
                buffer.extract_range(start.0, start.1, end.0, end.1 + 1)
            }
            VisualType::Line => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                let mut yanked = String::new();
                for row in start_row..=end_row {
                    if let Some(line) = buffer.content().get(row) {
                        yanked.push_str(line);
                        yanked.push('\n');
                    }
                }
                yanked
            }
            VisualType::Block => {
                let top_row = self.anchor.0.min(cursor.0);
                let bottom_row = self.anchor.0.max(cursor.0);
                let (left_display, right_display) =
                    Self::block_display_range(buffer, self.anchor, cursor);

                let mut yanked = String::new();
                for row in top_row..=bottom_row {
                    let (g_start, g_end) =
                        buffer.grapheme_range_for_display_cols(row, left_display, right_display);
                    let text = buffer.extract_range(row, g_start, row, g_end);
                    yanked.push_str(&text);
                    if row < bottom_row {
                        yanked.push('\n');
                    }
                }
                yanked
            }
        }
    }

    pub fn indent_selection(&self, buffer: &mut TextBuffer, cursor: (usize, usize), tab_width: u8) {
        match self.visual_type {
            VisualType::Line | VisualType::Block => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                buffer.indent_lines(start_row, end_row, tab_width);
            }
            VisualType::Character => {
                // For character mode, indent the lines that are partially selected
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                buffer.indent_lines(start_row, end_row, tab_width);
            }
        }
    }

    pub fn dedent_selection(&self, buffer: &mut TextBuffer, cursor: (usize, usize), tab_width: u8) {
        match self.visual_type {
            VisualType::Line | VisualType::Block => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                buffer.dedent_lines(start_row, end_row, tab_width);
            }
            VisualType::Character => {
                let start_row = self.anchor.0.min(cursor.0);
                let end_row = self.anchor.0.max(cursor.0);
                buffer.dedent_lines(start_row, end_row, tab_width);
            }
        }
    }

    pub fn toggle_comment(&self, buffer: &mut TextBuffer, cursor: (usize, usize)) {
        let start_row = self.anchor.0.min(cursor.0);
        let end_row = self.anchor.0.max(cursor.0);
        buffer.toggle_comment_lines(start_row, end_row);
    }

    pub fn prepare_insert_start(
        &mut self,
        buffer: &mut TextBuffer,
        cursor: (usize, usize),
    ) -> Vec<(usize, usize)> {
        if self.visual_type != VisualType::Block {
            return vec![];
        }

        let top_row = self.anchor.0.min(cursor.0);
        let bottom_row = self.anchor.0.max(cursor.0);
        let (left_display, _) = Self::block_display_range(buffer, self.anchor, cursor);

        let mut positions = Vec::new();
        for row in top_row..=bottom_row {
            let col = buffer.grapheme_at_display_col(row, left_display);
            positions.push((row, col));
        }
        positions
    }

    pub fn prepare_insert_end(
        &mut self,
        buffer: &mut TextBuffer,
        cursor: (usize, usize),
    ) -> Vec<(usize, usize)> {
        if self.visual_type != VisualType::Block {
            return vec![];
        }

        let top_row = self.anchor.0.min(cursor.0);
        let bottom_row = self.anchor.0.max(cursor.0);
        let (_, right_display) = Self::block_display_range(buffer, self.anchor, cursor);

        let mut positions = Vec::new();
        for row in top_row..=bottom_row {
            // Find grapheme just past the right edge of the block
            let (_, g_end) =
                buffer.grapheme_range_for_display_cols(row, right_display, right_display);
            let line_len = buffer
                .content()
                .get(row)
                .map(|l| l.graphemes(true).count())
                .unwrap_or(0);
            let insert_col = g_end.min(line_len);
            positions.push((row, insert_col));
        }
        positions
    }

    /// Compute the display column range for a block selection.
    ///
    /// Returns `(left_display, right_display)` where both are inclusive display
    /// column indices. The range encompasses the full display width of the
    /// characters at both anchor and cursor positions, ensuring wide characters
    /// (CJK) are never partially selected.
    fn block_display_range(
        buffer: &TextBuffer,
        anchor: (usize, usize),
        cursor: (usize, usize),
    ) -> (usize, usize) {
        let anchor_start = buffer.display_col_at(anchor.0, anchor.1);
        let anchor_width = buffer.grapheme_display_width(anchor.0, anchor.1);
        let anchor_end = anchor_start + anchor_width - 1;

        let cursor_start = buffer.display_col_at(cursor.0, cursor.1);
        let cursor_width = buffer.grapheme_display_width(cursor.0, cursor.1);
        let cursor_end = cursor_start + cursor_width - 1;

        let left = anchor_start.min(cursor_start);
        let right = anchor_end.max(cursor_end);

        (left, right)
    }

    fn normalize_range(
        anchor: (usize, usize),
        cursor: (usize, usize),
    ) -> ((usize, usize), (usize, usize)) {
        if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buffer(content: &str) -> TextBuffer {
        TextBuffer::from_string(content)
    }

    // ---- Construction & accessors ----

    #[test]
    fn test_new_character() {
        let vm = VisualMode::new(VisualType::Character, (1, 2));
        assert_eq!(vm.anchor(), (1, 2));
    }

    #[test]
    fn test_new_line() {
        let vm = VisualMode::new(VisualType::Line, (3, 0));
        assert_eq!(vm.anchor(), (3, 0));
    }

    #[test]
    fn test_new_block() {
        let vm = VisualMode::new(VisualType::Block, (0, 5));
        assert_eq!(vm.anchor(), (0, 5));
    }

    #[test]
    fn test_set_type() {
        let mut vm = VisualMode::new(VisualType::Character, (0, 0));
        vm.set_type(VisualType::Line);
        // After set_type, render_data should produce LineRange
        let buf = make_buffer("hello\nworld");
        let sel = vm.render_data(&buf, (1, 0));
        match sel {
            RenderSelection::LineRange { start_row, end_row } => {
                assert_eq!(start_row, 0);
                assert_eq!(end_row, 1);
            }
            _ => panic!("expected LineRange"),
        }
    }

    // ---- render_data: Character ----

    #[test]
    fn test_render_data_character_forward() {
        let vm = VisualMode::new(VisualType::Character, (0, 1));
        let buf = make_buffer("hello");
        match vm.render_data(&buf, (0, 3)) {
            RenderSelection::CharacterRange { start, end } => {
                assert_eq!(start, (0, 1));
                assert_eq!(end, (0, 3));
            }
            _ => panic!("expected CharacterRange"),
        }
    }

    #[test]
    fn test_render_data_character_backward() {
        let vm = VisualMode::new(VisualType::Character, (0, 4));
        let buf = make_buffer("hello");
        match vm.render_data(&buf, (0, 1)) {
            RenderSelection::CharacterRange { start, end } => {
                assert_eq!(start, (0, 1));
                assert_eq!(end, (0, 4));
            }
            _ => panic!("expected CharacterRange"),
        }
    }

    #[test]
    fn test_render_data_character_multiline() {
        let vm = VisualMode::new(VisualType::Character, (0, 2));
        let buf = make_buffer("hello\nworld");
        match vm.render_data(&buf, (1, 3)) {
            RenderSelection::CharacterRange { start, end } => {
                assert_eq!(start, (0, 2));
                assert_eq!(end, (1, 3));
            }
            _ => panic!("expected CharacterRange"),
        }
    }

    // ---- render_data: Line ----

    #[test]
    fn test_render_data_line_single() {
        let vm = VisualMode::new(VisualType::Line, (1, 0));
        let buf = make_buffer("aaa\nbbb\nccc");
        match vm.render_data(&buf, (1, 2)) {
            RenderSelection::LineRange { start_row, end_row } => {
                assert_eq!(start_row, 1);
                assert_eq!(end_row, 1);
            }
            _ => panic!("expected LineRange"),
        }
    }

    #[test]
    fn test_render_data_line_multi() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let buf = make_buffer("aaa\nbbb\nccc");
        match vm.render_data(&buf, (2, 0)) {
            RenderSelection::LineRange { start_row, end_row } => {
                assert_eq!(start_row, 0);
                assert_eq!(end_row, 2);
            }
            _ => panic!("expected LineRange"),
        }
    }

    // ---- render_data: Block ----

    #[test]
    fn test_render_data_block_basic() {
        let vm = VisualMode::new(VisualType::Block, (0, 1));
        let buf = make_buffer("abcde\nfghij");
        match vm.render_data(&buf, (1, 3)) {
            RenderSelection::BlockRegion {
                top_row,
                bottom_row,
                left_col,
                right_col,
            } => {
                assert_eq!(top_row, 0);
                assert_eq!(bottom_row, 1);
                assert_eq!(left_col, 1);
                assert_eq!(right_col, 3);
            }
            _ => panic!("expected BlockRegion"),
        }
    }

    #[test]
    fn test_render_data_block_cjk() {
        // CJK characters have display width 2
        let vm = VisualMode::new(VisualType::Block, (0, 0));
        let buf = make_buffer("你好世界\nabcdefgh");
        match vm.render_data(&buf, (1, 3)) {
            RenderSelection::BlockRegion {
                left_col,
                right_col,
                ..
            } => {
                // anchor 你 starts at display 0, width 2 → anchor_end = 1
                // cursor 'd' at grapheme 3, display 3, width 1 → cursor_end = 3
                assert_eq!(left_col, 0);
                assert_eq!(right_col, 3);
            }
            _ => panic!("expected BlockRegion"),
        }
    }

    // ---- yank_selection ----

    #[test]
    fn test_yank_character_single_line() {
        let vm = VisualMode::new(VisualType::Character, (0, 1));
        let buf = make_buffer("hello world");
        let yanked = vm.yank_selection(&buf, (0, 4));
        assert_eq!(yanked, "ello");
    }

    #[test]
    fn test_yank_character_multiline() {
        let vm = VisualMode::new(VisualType::Character, (0, 3));
        let buf = make_buffer("hello\nworld");
        let yanked = vm.yank_selection(&buf, (1, 2));
        assert_eq!(yanked, "lo\nwor");
    }

    #[test]
    fn test_yank_line_single() {
        let vm = VisualMode::new(VisualType::Line, (1, 0));
        let buf = make_buffer("aaa\nbbb\nccc");
        let yanked = vm.yank_selection(&buf, (1, 2));
        assert_eq!(yanked, "bbb\n");
    }

    #[test]
    fn test_yank_line_multi() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let buf = make_buffer("aaa\nbbb\nccc");
        let yanked = vm.yank_selection(&buf, (2, 0));
        assert_eq!(yanked, "aaa\nbbb\nccc\n");
    }

    #[test]
    fn test_yank_block_basic() {
        let vm = VisualMode::new(VisualType::Block, (0, 1));
        let buf = make_buffer("abcde\nfghij\nklmno");
        let yanked = vm.yank_selection(&buf, (2, 3));
        // Columns 1..3 inclusive (display cols 1..3)
        assert_eq!(yanked, "bcd\nghi\nlmn");
    }

    // ---- delete_selection ----

    #[test]
    fn test_delete_character_single_line() {
        let vm = VisualMode::new(VisualType::Character, (0, 1));
        let mut buf = make_buffer("hello");
        let deleted = vm.delete_selection(&mut buf, (0, 3));
        assert_eq!(deleted, "ell");
        assert_eq!(buf.to_string(), "ho");
    }

    #[test]
    fn test_delete_character_multiline() {
        let vm = VisualMode::new(VisualType::Character, (0, 3));
        let mut buf = make_buffer("hello\nworld");
        let deleted = vm.delete_selection(&mut buf, (1, 2));
        assert_eq!(deleted, "lo\nwor");
        assert_eq!(buf.to_string(), "helld");
    }

    #[test]
    fn test_delete_line_single() {
        let vm = VisualMode::new(VisualType::Line, (1, 0));
        let mut buf = make_buffer("aaa\nbbb\nccc");
        let deleted = vm.delete_selection(&mut buf, (1, 2));
        assert_eq!(deleted, "bbb\n");
        assert_eq!(buf.to_string(), "aaa\nccc");
    }

    #[test]
    fn test_delete_line_multi() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let mut buf = make_buffer("aaa\nbbb\nccc");
        let deleted = vm.delete_selection(&mut buf, (1, 0));
        assert_eq!(deleted, "aaa\nbbb\n");
        assert_eq!(buf.to_string(), "ccc");
    }

    #[test]
    fn test_delete_block_basic() {
        let vm = VisualMode::new(VisualType::Block, (0, 1));
        let mut buf = make_buffer("abcde\nfghij");
        let _deleted = vm.delete_selection(&mut buf, (1, 3));
        assert_eq!(buf.content()[0], "ae");
        assert_eq!(buf.content()[1], "fj");
    }

    // ---- indent / dedent ----

    #[test]
    fn test_indent_selection_line_mode() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let mut buf = make_buffer("aaa\nbbb");
        vm.indent_selection(&mut buf, (1, 0), 4);
        assert!(buf.content()[0].starts_with("    "));
        assert!(buf.content()[1].starts_with("    "));
    }

    #[test]
    fn test_indent_selection_character_mode() {
        let vm = VisualMode::new(VisualType::Character, (0, 0));
        let mut buf = make_buffer("aaa\nbbb");
        vm.indent_selection(&mut buf, (1, 0), 2);
        assert!(buf.content()[0].starts_with("  "));
        assert!(buf.content()[1].starts_with("  "));
    }

    #[test]
    fn test_dedent_selection_line_mode() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let mut buf = make_buffer("    aaa\n    bbb");
        vm.dedent_selection(&mut buf, (1, 4), 4);
        assert_eq!(buf.content()[0], "aaa");
        assert_eq!(buf.content()[1], "bbb");
    }

    #[test]
    fn test_dedent_selection_partial() {
        let vm = VisualMode::new(VisualType::Line, (0, 0));
        let mut buf = make_buffer("  aaa\n  bbb");
        vm.dedent_selection(&mut buf, (1, 2), 4);
        assert_eq!(buf.content()[0], "aaa");
        assert_eq!(buf.content()[1], "bbb");
    }

    // ---- toggle_comment ----

    #[test]
    fn test_toggle_comment_range() {
        let vm = VisualMode::new(VisualType::Character, (0, 0));
        let mut buf = make_buffer("hello\nworld");
        vm.toggle_comment(&mut buf, (1, 0));
        assert_eq!(buf.content()[0], "<!-- hello -->");
        assert_eq!(buf.content()[1], "<!-- world -->");
    }

    // ---- prepare_insert_start / prepare_insert_end ----

    #[test]
    fn test_prepare_insert_start_block() {
        let mut vm = VisualMode::new(VisualType::Block, (0, 1));
        let mut buf = make_buffer("abcde\nfghij\nklmno");
        let positions = vm.prepare_insert_start(&mut buf, (2, 3));
        assert_eq!(positions.len(), 3);
        // All should be at grapheme index for display col 1
        assert_eq!(positions[0].1, 1);
        assert_eq!(positions[1].1, 1);
        assert_eq!(positions[2].1, 1);
    }

    #[test]
    fn test_prepare_insert_end_block() {
        let mut vm = VisualMode::new(VisualType::Block, (0, 1));
        let mut buf = make_buffer("abcde\nfghij\nklmno");
        let positions = vm.prepare_insert_end(&mut buf, (2, 3));
        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn test_prepare_insert_start_non_block_returns_empty() {
        let mut vm = VisualMode::new(VisualType::Character, (0, 0));
        let mut buf = make_buffer("hello");
        let positions = vm.prepare_insert_start(&mut buf, (0, 3));
        assert!(positions.is_empty());
    }

    #[test]
    fn test_prepare_insert_end_non_block_returns_empty() {
        let mut vm = VisualMode::new(VisualType::Line, (0, 0));
        let mut buf = make_buffer("hello");
        let positions = vm.prepare_insert_end(&mut buf, (0, 3));
        assert!(positions.is_empty());
    }

    // ---- CJK-specific tests ----

    #[test]
    fn test_yank_character_cjk() {
        let vm = VisualMode::new(VisualType::Character, (0, 0));
        let buf = make_buffer("你好世界");
        let yanked = vm.yank_selection(&buf, (0, 2));
        assert_eq!(yanked, "你好世");
    }

    #[test]
    fn test_delete_character_cjk() {
        let vm = VisualMode::new(VisualType::Character, (0, 1));
        let mut buf = make_buffer("你好世界");
        let deleted = vm.delete_selection(&mut buf, (0, 2));
        assert_eq!(deleted, "好世");
        assert_eq!(buf.to_string(), "你界");
    }
}

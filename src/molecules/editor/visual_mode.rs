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
                let (left_col, right_col) =
                    Self::block_display_range(buffer, self.anchor, cursor);
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

    pub fn dedent_selection(
        &self,
        buffer: &mut TextBuffer,
        cursor: (usize, usize),
        tab_width: u8,
    ) {
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

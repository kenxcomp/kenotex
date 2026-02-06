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
        left_col: usize,
        right_col: usize,
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

    pub fn render_data(&self, cursor: (usize, usize)) -> RenderSelection {
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
                let left_col = self.anchor.1.min(cursor.1);
                let right_col = self.anchor.1.max(cursor.1);
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
                let left_col = self.anchor.1.min(cursor.1);
                let right_col = self.anchor.1.max(cursor.1);

                let mut deleted = String::new();
                for row in (top_row..=bottom_row).rev() {
                    let text = buffer.delete_range(row, left_col, row, right_col + 1);
                    if row > top_row {
                        deleted.insert_str(0, &format!("\n{}", text));
                    } else {
                        deleted.insert_str(0, &text);
                    }
                }
                buffer.set_cursor(top_row, left_col);
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
                let left_col = self.anchor.1.min(cursor.1);
                let right_col = self.anchor.1.max(cursor.1);

                let mut yanked = String::new();
                for row in top_row..=bottom_row {
                    let text = buffer.extract_range(row, left_col, row, right_col + 1);
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
        let left_col = self.anchor.1.min(cursor.1);

        let mut positions = Vec::new();
        for row in top_row..=bottom_row {
            positions.push((row, left_col));
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
        let right_col = self.anchor.1.max(cursor.1) + 1;

        let mut positions = Vec::new();
        for row in top_row..=bottom_row {
            // Get actual line length
            let line_len = buffer
                .content()
                .get(row)
                .map(|l| l.graphemes(true).count())
                .unwrap_or(0);
            let insert_col = right_col.min(line_len);
            positions.push((row, insert_col));
        }
        positions
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

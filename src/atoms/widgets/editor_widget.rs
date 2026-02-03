use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use regex::Regex;

use crate::types::{AppMode, Theme};

pub struct EditorWidget<'a> {
    content: &'a str,
    cursor_pos: (usize, usize),
    theme: &'a Theme,
    mode: AppMode,
    title: &'a str,
    scroll_offset: u16,
    visual_selection: Option<((usize, usize), (usize, usize))>,
}

impl<'a> EditorWidget<'a> {
    pub fn new(
        content: &'a str,
        cursor_pos: (usize, usize),
        theme: &'a Theme,
        mode: AppMode,
        title: &'a str,
    ) -> Self {
        Self {
            content,
            cursor_pos,
            theme,
            mode,
            title,
            scroll_offset: 0,
            visual_selection: None,
        }
    }

    pub fn scroll_offset(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn visual_selection(mut self, sel: Option<((usize, usize), (usize, usize))>) -> Self {
        self.visual_selection = sel;
        self
    }

    fn highlight_line(&self, line: &str, line_idx: usize) -> Line<'a> {
        let mut spans = Vec::new();
        let heading_re = Regex::new(r"^(#{1,6})\s+(.*)$").unwrap();
        let checkbox_unchecked_re = Regex::new(r"^(\s*-\s*\[\s*\])\s*(.*)$").unwrap();
        let checkbox_checked_re = Regex::new(r"^(\s*-\s*\[x\])\s*(.*)$").unwrap();
        let smart_tag_re = Regex::new(r"(:::(?:td|cal|note))\s*(.*)$").unwrap();

        let is_cursor_line = line_idx == self.cursor_pos.0;
        let base_style = if is_cursor_line && self.mode == AppMode::Normal {
            Style::default()
                .fg(self.theme.fg_color())
                .bg(self.theme.selection_color())
        } else {
            Style::default().fg(self.theme.fg_color())
        };

        if let Some(caps) = heading_re.captures(line) {
            let hashes = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            spans.push(Span::styled(
                hashes.to_string() + " ",
                base_style
                    .fg(self.theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                text.to_string(),
                base_style
                    .fg(self.theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ));
        } else if let Some(caps) = checkbox_checked_re.captures(line) {
            let checkbox = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            spans.push(Span::styled(
                checkbox.to_string() + " ",
                base_style
                    .fg(self.theme.success_color())
                    .add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
            ));
            spans.push(Span::styled(
                text.to_string(),
                base_style
                    .fg(self.theme.fg_color())
                    .add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
            ));
        } else if let Some(caps) = checkbox_unchecked_re.captures(line) {
            let checkbox = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            spans.push(Span::styled(
                checkbox.to_string() + " ",
                base_style.fg(self.theme.warning_color()),
            ));
            spans.push(Span::styled(text.to_string(), base_style));
        } else if let Some(caps) = smart_tag_re.captures(line) {
            let tag = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            spans.push(Span::styled(
                tag.to_string() + " ",
                base_style
                    .fg(self.theme.error_color())
                    .add_modifier(Modifier::ITALIC),
            ));
            spans.push(Span::styled(text.to_string(), base_style));
        } else {
            spans.push(Span::styled(line.to_string(), base_style));
        }

        Line::from(spans)
    }
}

impl Widget for EditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_color()))
            .title(format!(" {} ", self.title))
            .title_style(Style::default().fg(self.theme.accent_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let lines: Vec<Line> = self
            .content
            .lines()
            .enumerate()
            .map(|(idx, line)| self.highlight_line(line, idx))
            .collect();

        let paragraph = Paragraph::new(lines)
            .style(Style::default().bg(self.theme.bg_color()))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        paragraph.render(inner, buf);

        // Render visual selection highlight
        if let Some(((sr, sc), (er, ec))) = self.visual_selection {
            use super::wrap_calc;
            use unicode_segmentation::UnicodeSegmentation;

            let selection_style = Style::default()
                .bg(self.theme.accent_color())
                .fg(self.theme.bg_color());

            let content_lines: Vec<String> = self.content.lines().map(String::from).collect();

            // Compute rows_before for the first selected line
            let mut rows_before: u16 = content_lines
                .iter()
                .take(sr)
                .map(|l| wrap_calc::display_rows_for_line(l, inner.width))
                .sum();

            for row in sr..=er {
                let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
                let graphemes: Vec<&str> = line.graphemes(true).collect();

                let col_start = if row == sr { sc } else { 0 };
                let col_end = if row == er { ec + 1 } else { graphemes.len() + 1 };
                let col_end = col_end.min(graphemes.len() + 1);

                let positions =
                    wrap_calc::visual_positions_in_range(line, col_start, col_end, inner.width);

                for (wrap_row, col, gw) in positions {
                    let screen_y =
                        inner.y + rows_before + wrap_row - self.scroll_offset;
                    if screen_y < inner.y || screen_y >= inner.y + inner.height {
                        continue;
                    }
                    let screen_x = inner.x + col;
                    for dx in 0..gw {
                        if screen_x + dx < inner.x + inner.width {
                            buf[(screen_x + dx, screen_y)].set_style(selection_style);
                        }
                    }
                }

                rows_before += wrap_calc::display_rows_for_line(line, inner.width);
            }
        }

        // Render block cursor only in Normal mode
        // Insert mode uses native terminal cursor (I-beam) set in main.rs
        if self.mode == AppMode::Normal {
            use super::wrap_calc;

            let cursor_row = self.cursor_pos.0;
            let cursor_col = self.cursor_pos.1;

            let content_lines: Vec<String> =
                self.content.lines().map(String::from).collect();
            let vpos =
                wrap_calc::visual_cursor_position(&content_lines, cursor_row, cursor_col, inner.width);

            let cursor_x = inner.x + vpos.col;
            let cursor_y =
                inner.y + vpos.rows_before + vpos.wrap_row - self.scroll_offset;

            if cursor_y >= inner.y
                && cursor_y < inner.y + inner.height
                && cursor_x < inner.x + inner.width
            {
                let cursor_style = Style::default()
                    .fg(self.theme.bg_color())
                    .bg(self.theme.cursor_color());
                buf[(cursor_x, cursor_y)].set_style(cursor_style);
            }
        }
    }
}

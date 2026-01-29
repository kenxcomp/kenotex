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
        }
    }

    pub fn scroll_offset(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
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

        if self.mode == AppMode::Insert {
            let cursor_x = inner.x + self.cursor_pos.1 as u16;
            let cursor_y = inner.y + self.cursor_pos.0 as u16 - self.scroll_offset;

            if cursor_y >= inner.y && cursor_y < inner.y + inner.height {
                if cursor_x < inner.x + inner.width {
                    buf[(cursor_x, cursor_y)]
                        .set_style(Style::default().bg(self.theme.cursor_color()));
                }
            }
        }
    }
}

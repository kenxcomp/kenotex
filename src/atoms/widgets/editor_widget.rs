use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use regex::Regex;
use std::sync::LazyLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::types::{AppMode, Theme};

use super::md_highlight::{MdTokenKind, tokenize_inline};

// Cached regex patterns for syntax highlighting
static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#{1,6}\s").unwrap());
static CHECKBOX_CHECKED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*-\s*\[x\]\s?").unwrap());
static CHECKBOX_UNCHECKED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*-\s*\[\s*\]\s?").unwrap());
static SMART_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:::(?:td|cal|note)\s?").unwrap());

/// Split a styled `Line` into multiple display lines using character-level wrapping.
///
/// This must match the algorithm in `wrap_calc` exactly so that cursor position
/// calculations agree with the rendered text. Ratatui's `Wrap { trim: false }` uses
/// word-boundary wrapping which produces different break points, causing cursor drift.
fn split_line_by_width(line: Line<'_>, width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        let spans: Vec<Span<'static>> = line
            .spans
            .into_iter()
            .map(|s| Span::styled(s.content.into_owned(), s.style))
            .collect();
        return vec![Line::from(spans)];
    }

    let max_w = width as usize;

    // Flatten spans into (grapheme_string, style, display_width)
    let mut graphemes: Vec<(String, Style, usize)> = Vec::new();
    for span in line.spans {
        let style = span.style;
        for g in span.content.graphemes(true) {
            graphemes.push((g.to_string(), style, g.width()));
        }
    }

    if graphemes.is_empty() {
        return vec![Line::from("")];
    }

    // Split into display lines at character-level wrap points
    let mut result: Vec<Line<'static>> = Vec::new();
    let mut col: usize = 0;
    let mut line_start: usize = 0;

    for i in 0..graphemes.len() {
        let gw = graphemes[i].2;
        if gw > 0 && col + gw > max_w {
            result.push(build_display_line(&graphemes[line_start..i]));
            line_start = i;
            col = 0;
        }
        col += gw;
    }

    // Last segment
    result.push(build_display_line(&graphemes[line_start..]));

    result
}

/// Re-merge consecutive same-style graphemes into Spans to form a display Line.
fn build_display_line(graphemes: &[(String, Style, usize)]) -> Line<'static> {
    if graphemes.is_empty() {
        return Line::from("");
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style = graphemes[0].1;

    for (g, style, _) in graphemes {
        if *style == current_style {
            current_text.push_str(g);
        } else {
            spans.push(Span::styled(
                std::mem::take(&mut current_text),
                current_style,
            ));
            current_text.push_str(g);
            current_style = *style;
        }
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    Line::from(spans)
}

/// Pre-scan all lines to determine which are inside code block fences (```).
fn compute_code_block_flags(content: &str) -> Vec<bool> {
    let lines: Vec<&str> = content.lines().collect();
    let mut flags = vec![false; lines.len()];
    let mut in_code_block = false;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flags[idx] = true; // fence line itself is marked
            in_code_block = !in_code_block;
        } else if in_code_block {
            flags[idx] = true;
        }
    }

    flags
}

pub struct EditorWidget<'a> {
    content: &'a str,
    cursor_pos: (usize, usize),
    theme: &'a Theme,
    mode: AppMode,
    title: &'a str,
    scroll_offset: u16,
    visual_selection: Option<((usize, usize), (usize, usize))>,
    search_matches: &'a [(usize, usize, usize)],
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
            search_matches: &[],
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

    pub fn search_matches(mut self, matches: &'a [(usize, usize, usize)]) -> Self {
        self.search_matches = matches;
        self
    }

    /// Map MdTokenKind to ratatui Style based on theme colors.
    fn style_for_token(&self, kind: &MdTokenKind, base_style: Style) -> Style {
        match kind {
            MdTokenKind::Plain => base_style,
            MdTokenKind::Bold => base_style
                .fg(self.theme.accent_color())
                .add_modifier(Modifier::BOLD),
            MdTokenKind::Italic => base_style.add_modifier(Modifier::ITALIC),
            MdTokenKind::BoldItalic => base_style
                .fg(self.theme.accent_color())
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            MdTokenKind::Strikethrough => {
                base_style.add_modifier(Modifier::DIM | Modifier::CROSSED_OUT)
            }
            MdTokenKind::InlineCode => base_style
                .fg(self.theme.warning_color())
                .bg(self.theme.panel_color()),
            MdTokenKind::Delimiter => base_style.add_modifier(Modifier::DIM),
            MdTokenKind::OrderedListPrefix | MdTokenKind::UnorderedListPrefix => {
                base_style.fg(self.theme.border_color())
            }
        }
    }

    fn highlight_line(&self, line: &str, line_idx: usize, in_code_block: bool) -> Line<'a> {
        let mut spans = Vec::new();

        let is_cursor_line = line_idx == self.cursor_pos.0;
        let base_style = if is_cursor_line && self.mode == AppMode::Normal {
            Style::default()
                .fg(self.theme.fg_color())
                .bg(self.theme.selection_color())
        } else {
            Style::default().fg(self.theme.fg_color())
        };

        // Code block handling
        if in_code_block {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                // Fence line: border color + dim
                spans.push(Span::styled(
                    line.to_string(),
                    base_style
                        .fg(self.theme.border_color())
                        .bg(self.theme.panel_color())
                        .add_modifier(Modifier::DIM),
                ));
            } else {
                // Code block content: panel background
                spans.push(Span::styled(
                    line.to_string(),
                    base_style.bg(self.theme.panel_color()),
                ));
            }
            return Line::from(spans);
        }

        // Line-level patterns (heading, checkbox, smart tag)
        if let Some(m) = HEADING_RE.find(line) {
            let prefix = &line[..m.end()];
            let rest = &line[m.end()..];

            // Heading prefix with accent + bold
            spans.push(Span::styled(
                prefix.to_string(),
                base_style
                    .fg(self.theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ));

            // Rest portion: tokenize inline with accent + bold as base
            let heading_base = base_style
                .fg(self.theme.accent_color())
                .add_modifier(Modifier::BOLD);
            for token in tokenize_inline(rest) {
                let token_style = self.style_for_token(&token.kind, heading_base);
                spans.push(Span::styled(token.text, token_style));
            }
        } else if let Some(m) = CHECKBOX_CHECKED_RE.find(line) {
            let prefix = &line[..m.end()];
            let rest = &line[m.end()..];

            // Checkbox prefix
            spans.push(Span::styled(
                prefix.to_string(),
                base_style
                    .fg(self.theme.success_color())
                    .add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
            ));

            // Rest portion: tokenize inline with dimmed strikethrough as base
            let checkbox_base = base_style.add_modifier(Modifier::DIM | Modifier::CROSSED_OUT);
            for token in tokenize_inline(rest) {
                let token_style = self.style_for_token(&token.kind, checkbox_base);
                spans.push(Span::styled(token.text, token_style));
            }
        } else if let Some(m) = CHECKBOX_UNCHECKED_RE.find(line) {
            let prefix = &line[..m.end()];
            let rest = &line[m.end()..];

            // Checkbox prefix
            spans.push(Span::styled(
                prefix.to_string(),
                base_style.fg(self.theme.warning_color()),
            ));

            // Rest portion: tokenize inline
            for token in tokenize_inline(rest) {
                let token_style = self.style_for_token(&token.kind, base_style);
                spans.push(Span::styled(token.text, token_style));
            }
        } else if let Some(m) = SMART_TAG_RE.find(line) {
            let prefix = &line[..m.end()];
            let rest = &line[m.end()..];

            // Smart tag prefix
            spans.push(Span::styled(
                prefix.to_string(),
                base_style
                    .fg(self.theme.error_color())
                    .add_modifier(Modifier::ITALIC),
            ));

            // Rest portion: tokenize inline
            for token in tokenize_inline(rest) {
                let token_style = self.style_for_token(&token.kind, base_style);
                spans.push(Span::styled(token.text, token_style));
            }
        } else {
            // Plain line: tokenize inline
            for token in tokenize_inline(line) {
                let token_style = self.style_for_token(&token.kind, base_style);
                spans.push(Span::styled(token.text, token_style));
            }
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

        // Pre-compute code block flags
        let code_block_flags = compute_code_block_flags(self.content);

        // Pre-split styled lines using character-level wrapping so that
        // the rendered text matches wrap_calc's cursor position calculations.
        let display_lines: Vec<Line> = self
            .content
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                let in_code_block = code_block_flags.get(idx).copied().unwrap_or(false);
                self.highlight_line(line, idx, in_code_block)
            })
            .flat_map(|line| split_line_by_width(line, inner.width))
            .collect();

        let paragraph = Paragraph::new(display_lines)
            .style(Style::default().bg(self.theme.bg_color()))
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
                let col_end = if row == er {
                    ec + 1
                } else {
                    graphemes.len() + 1
                };
                let col_end = col_end.min(graphemes.len() + 1);

                let positions =
                    wrap_calc::visual_positions_in_range(line, col_start, col_end, inner.width);

                for (wrap_row, col, gw) in positions {
                    let screen_y = inner.y + rows_before + wrap_row - self.scroll_offset;
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

        // Render search match highlights
        if !self.search_matches.is_empty() {
            use super::wrap_calc;

            let search_style = Style::default()
                .bg(self.theme.warning_color())
                .fg(self.theme.bg_color());

            let content_lines: Vec<String> = self.content.lines().map(String::from).collect();

            for &(match_row, match_col, match_len) in self.search_matches {
                if match_row >= content_lines.len() {
                    continue;
                }

                let rows_before: u16 = content_lines
                    .iter()
                    .take(match_row)
                    .map(|l| wrap_calc::display_rows_for_line(l, inner.width))
                    .sum();

                let line = &content_lines[match_row];
                let positions = wrap_calc::visual_positions_in_range(
                    line,
                    match_col,
                    match_col + match_len,
                    inner.width,
                );

                for (wrap_row, col, gw) in positions {
                    let screen_y = inner.y + rows_before + wrap_row - self.scroll_offset;
                    if screen_y < inner.y || screen_y >= inner.y + inner.height {
                        continue;
                    }
                    let screen_x = inner.x + col;
                    for dx in 0..gw {
                        if screen_x + dx < inner.x + inner.width {
                            buf[(screen_x + dx, screen_y)].set_style(search_style);
                        }
                    }
                }
            }
        }

        // Render block cursor only in Normal mode
        // Insert mode uses native terminal cursor (I-beam) set in main.rs
        if self.mode == AppMode::Normal {
            use super::wrap_calc;

            let cursor_row = self.cursor_pos.0;
            let cursor_col = self.cursor_pos.1;

            let content_lines: Vec<String> = self.content.lines().map(String::from).collect();
            let vpos = wrap_calc::visual_cursor_position(
                &content_lines,
                cursor_row,
                cursor_col,
                inner.width,
            );

            let cursor_x = inner.x + vpos.col;
            let cursor_y = inner.y + vpos.rows_before + vpos.wrap_row - self.scroll_offset;

            if cursor_y >= inner.y
                && cursor_y < inner.y + inner.height
                && cursor_x < inner.x + inner.width
            {
                let cursor_style = Style::default()
                    .fg(self.theme.bg_color())
                    .bg(self.theme.cursor_color());

                // Determine display width of character under cursor (CJK = 2 cells)
                let char_width = content_lines
                    .get(cursor_row)
                    .and_then(|line| line.graphemes(true).nth(cursor_col))
                    .map(|g| g.width().max(1))
                    .unwrap_or(1) as u16;

                for dx in 0..char_width {
                    if cursor_x + dx < inner.x + inner.width {
                        buf[(cursor_x + dx, cursor_y)].set_style(cursor_style);
                    }
                }
            }
        }
    }
}

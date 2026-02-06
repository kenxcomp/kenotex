use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use regex::Regex;
use std::sync::LazyLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::molecules::editor::RenderSelection;
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
    visual_selection: Option<RenderSelection>,
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

    pub fn visual_selection(mut self, sel: Option<RenderSelection>) -> Self {
        self.visual_selection = sel;
        self
    }

    pub fn search_matches(mut self, matches: &'a [(usize, usize, usize)]) -> Self {
        self.search_matches = matches;
        self
    }

    /// Applies selection background while preserving markdown formatting.
    ///
    /// Ratatui's `Cell::set_style()` replaces the entire style, so we must manually
    /// preserve formatting from the original cell. This function:
    /// - Preserves all text modifiers (BOLD, ITALIC, CROSSED_OUT, etc.)
    /// - Preserves foreground color (for markdown syntax highlighting)
    /// - Overrides background color only (for selection visibility)
    ///
    /// # Arguments
    /// * `buf` - The buffer to modify
    /// * `x`, `y` - Coordinates of the cell to modify
    /// * `selection_bg` - Background color for selection
    /// * `default_fg` - Foreground color to use if cell has no custom foreground
    fn apply_selection_to_cell(
        buf: &mut Buffer,
        x: u16,
        y: u16,
        selection_bg: Color,
        default_fg: Color,
    ) {
        let cell = &buf[(x, y)];
        let old_fg = cell.fg;
        let old_modifiers = cell.modifier;

        // Use the cell's foreground color if set, otherwise use default
        let fg_color = if matches!(old_fg, Color::Reset) {
            default_fg
        } else {
            old_fg
        };

        let new_style = Style::default()
            .bg(selection_bg)
            .fg(fg_color)
            .add_modifier(old_modifiers);

        buf[(x, y)].set_style(new_style);
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

    fn render_selection(&self, selection: &RenderSelection, inner: Rect, buf: &mut Buffer) {
        match selection {
            RenderSelection::CharacterRange { start, end } => {
                self.render_character_selection(*start, *end, inner, buf);
            }
            RenderSelection::LineRange { start_row, end_row } => {
                self.render_line_selection(*start_row, *end_row, inner, buf);
            }
            RenderSelection::BlockRegion {
                top_row,
                bottom_row,
                left_col,
                right_col,
            } => {
                self.render_block_selection(
                    *top_row,
                    *bottom_row,
                    *left_col,
                    *right_col,
                    inner,
                    buf,
                );
            }
        }
    }

    fn render_character_selection(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        inner: Rect,
        buf: &mut Buffer,
    ) {
        use super::wrap_calc;
        use unicode_segmentation::UnicodeSegmentation;

        let (sr, sc) = start;
        let (er, ec) = end;

        let content_lines: Vec<String> = self.content.lines().map(String::from).collect();

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
                        Self::apply_selection_to_cell(
                            buf,
                            screen_x + dx,
                            screen_y,
                            self.theme.accent_color(),
                            self.theme.fg_color(),
                        );
                    }
                }
            }

            rows_before += wrap_calc::display_rows_for_line(line, inner.width);
        }
    }

    fn render_line_selection(
        &self,
        start_row: usize,
        end_row: usize,
        inner: Rect,
        buf: &mut Buffer,
    ) {
        use super::wrap_calc;

        let content_lines: Vec<String> = self.content.lines().map(String::from).collect();

        let mut rows_before: u16 = content_lines
            .iter()
            .take(start_row)
            .map(|l| wrap_calc::display_rows_for_line(l, inner.width))
            .sum();

        for row in start_row..=end_row {
            let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
            let num_display_rows = wrap_calc::display_rows_for_line(line, inner.width);

            for wrap_row in 0..num_display_rows {
                let screen_y = inner.y + rows_before + wrap_row - self.scroll_offset;
                if screen_y >= inner.y && screen_y < inner.y + inner.height {
                    // Highlight entire line width
                    for x in 0..inner.width {
                        Self::apply_selection_to_cell(
                            buf,
                            inner.x + x,
                            screen_y,
                            self.theme.accent_color(),
                            self.theme.fg_color(),
                        );
                    }
                }
            }

            rows_before += num_display_rows;
        }
    }

    /// Render block selection highlighting.
    ///
    /// `left_col` and `right_col` are display columns (inclusive), not grapheme indices.
    /// For each line, we find graphemes that overlap the display column range
    /// and highlight their cells. Wide characters (CJK) that partially overlap
    /// the boundary are fully included.
    fn render_block_selection(
        &self,
        top_row: usize,
        bottom_row: usize,
        left_col: usize,
        right_col: usize,
        inner: Rect,
        buf: &mut Buffer,
    ) {
        use super::wrap_calc;
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;

        let content_lines: Vec<String> = self.content.lines().map(String::from).collect();
        let w = if inner.width == 0 { 1 } else { inner.width as usize };

        let mut rows_before: u16 = content_lines
            .iter()
            .take(top_row)
            .map(|l| wrap_calc::display_rows_for_line(l, inner.width))
            .sum();

        for row in top_row..=bottom_row {
            let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
            let graphemes: Vec<&str> = line.graphemes(true).collect();

            // Walk graphemes, tracking display position and wrap state,
            // highlighting cells that overlap [left_col, right_col].
            let mut wrap_row: u16 = 0;
            let mut display_col: usize = 0;

            for g in &graphemes {
                let gw = g.width().max(1);

                // Check for wrap
                if display_col + gw > w {
                    wrap_row += 1;
                    display_col = 0;
                }

                let g_end = display_col + gw - 1; // inclusive end

                // Check if grapheme overlaps [left_col, right_col]
                if display_col <= right_col && g_end >= left_col {
                    let screen_y = inner.y + rows_before + wrap_row - self.scroll_offset;
                    if screen_y >= inner.y && screen_y < inner.y + inner.height {
                        for dx in 0..gw {
                            let screen_x = inner.x + display_col as u16 + dx as u16;
                            if screen_x < inner.x + inner.width {
                                Self::apply_selection_to_cell(
                                    buf,
                                    screen_x,
                                    screen_y,
                                    self.theme.accent_color(),
                                    self.theme.fg_color(),
                                );
                            }
                        }
                    }
                }

                display_col += gw;
            }

            // Handle virtual spaces beyond line end
            let line_display_width = display_col;
            if right_col >= line_display_width {
                let virtual_start = left_col.max(line_display_width);
                for dcol in virtual_start..=right_col {
                    // Track wrapping for virtual positions
                    if dcol > 0 && dcol >= w && (dcol % w == 0) {
                        wrap_row += 1;
                        // Virtual position wrapping resets to column 0
                    }
                    let col_in_row = dcol % w;
                    let screen_y = inner.y + rows_before + wrap_row - self.scroll_offset;
                    if screen_y >= inner.y && screen_y < inner.y + inner.height {
                        let screen_x = inner.x + col_in_row as u16;
                        if screen_x < inner.x + inner.width {
                            buf[(screen_x, screen_y)].set_char(' ');
                            Self::apply_selection_to_cell(
                                buf,
                                screen_x,
                                screen_y,
                                self.theme.accent_color(),
                                self.theme.fg_color(),
                            );
                        }
                    }
                }
            }

            rows_before += wrap_calc::display_rows_for_line(line, inner.width);
        }
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
        if let Some(ref selection) = self.visual_selection {
            self.render_selection(selection, inner, buf);
        }

        // Render search match highlights
        if !self.search_matches.is_empty() {
            use super::wrap_calc;

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
                            Self::apply_selection_to_cell(
                                buf,
                                screen_x + dx,
                                screen_y,
                                self.theme.warning_color(),
                                self.theme.fg_color(),
                            );
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

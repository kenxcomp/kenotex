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
use super::syntax_highlight::{CodeParseState, SyntaxHighlighter, SyntaxTokenKind};

// Cached regex patterns for syntax highlighting
static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#{1,6}\s").unwrap());
static CHECKBOX_CHECKED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*-\s*\[x\]\s?").unwrap());
static CHECKBOX_UNCHECKED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*-\s*\[\s*\]\s?").unwrap());
static SMART_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:::(?:td|cal|note)\s?").unwrap());
static CLOSING_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:::\s*$").unwrap());

/// Split a styled `Line` into multiple display lines using character-level wrapping.
///
/// This must match the algorithm in `wrap_calc` exactly so that cursor position
/// calculations agree with the rendered text. Ratatui's `Wrap { trim: false }` uses
/// word-boundary wrapping which produces different break points, causing cursor drift.
///
/// When `hanging_indent > 0`, continuation rows are prefixed with `hanging_indent`
/// spaces (plain style) and text wraps at `width - hanging_indent`.
fn split_line_by_width(
    line: Line<'_>,
    width: u16,
    hanging_indent: u16,
    default_style: Style,
) -> Vec<Line<'static>> {
    if width == 0 {
        let spans: Vec<Span<'static>> = line
            .spans
            .into_iter()
            .map(|s| Span::styled(s.content.into_owned(), s.style))
            .collect();
        return vec![Line::from(spans)];
    }

    let hi = if hanging_indent >= width {
        0
    } else {
        hanging_indent as usize
    };
    let max_w = width as usize;
    let ew = max_w - hi; // effective width for continuation rows

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
        let current_max = if result.is_empty() { max_w } else { ew };
        if gw > 0 && col + gw > current_max {
            result.push(build_display_line(&graphemes[line_start..i]));
            line_start = i;
            col = 0;
        }
        col += gw;
    }

    // Last segment
    let last_line = build_display_line(&graphemes[line_start..]);

    // Prepend hanging indent spaces to continuation lines (index >= 1)
    if hi > 0 && !result.is_empty() {
        let indent_span = Span::styled(" ".repeat(hi), default_style);
        let mut new_result = Vec::with_capacity(result.len() + 1);
        new_result.push(result.remove(0)); // row 0 stays as is
        for display_line in result {
            let mut spans = vec![indent_span.clone()];
            spans.extend(display_line.spans);
            new_result.push(Line::from(spans));
        }
        // Add indent to last line too if it's a continuation
        let mut last_spans = vec![indent_span];
        last_spans.extend(last_line.spans);
        new_result.push(Line::from(last_spans));
        new_result
    } else {
        result.push(last_line);
        result
    }
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

struct CodeBlockInfo {
    in_code_block: bool,
    language: Option<String>,
}

/// Pre-scan all lines to determine which are inside code block fences (```).
/// Also extracts language identifier from opening fences.
fn compute_code_block_info(content: &str) -> Vec<CodeBlockInfo> {
    let lines: Vec<&str> = content.lines().collect();
    let mut infos = Vec::with_capacity(lines.len());
    let mut in_code_block = false;
    let mut current_lang: Option<String> = None;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code_block {
                // Closing fence
                infos.push(CodeBlockInfo {
                    in_code_block: true,
                    language: current_lang.clone(),
                });
                in_code_block = false;
                current_lang = None;
            } else {
                // Opening fence - extract language
                let after_backticks = trimmed.trim_start_matches('`');
                let lang = after_backticks.split_whitespace().next();
                let lang = lang.filter(|l| !l.is_empty()).map(|l| l.to_string());
                current_lang = lang.clone();
                infos.push(CodeBlockInfo {
                    in_code_block: true,
                    language: lang,
                });
                in_code_block = true;
            }
        } else if in_code_block {
            infos.push(CodeBlockInfo {
                in_code_block: true,
                language: current_lang.clone(),
            });
        } else {
            infos.push(CodeBlockInfo {
                in_code_block: false,
                language: None,
            });
        }
    }

    infos
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
    hanging_indents: &'a [u16],
    syntax_highlighter: Option<&'a SyntaxHighlighter>,
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
            hanging_indents: &[],
            syntax_highlighter: None,
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

    pub fn hanging_indents(mut self, indents: &'a [u16]) -> Self {
        self.hanging_indents = indents;
        self
    }

    pub fn syntax_highlighter(mut self, highlighter: &'a SyntaxHighlighter) -> Self {
        self.syntax_highlighter = Some(highlighter);
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
                .fg(self.theme.string_color())
                .bg(self.theme.panel_color()),
            MdTokenKind::Delimiter => base_style.fg(self.theme.comment_color()),
            MdTokenKind::OrderedListPrefix | MdTokenKind::UnorderedListPrefix => {
                base_style.fg(self.theme.border_color())
            }
            MdTokenKind::TimeExpression => base_style
                .fg(self.theme.constant_color())
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Map SyntaxTokenKind to ratatui Style based on theme colors.
    fn syntax_token_style(&self, kind: &SyntaxTokenKind, base_style: Style) -> Style {
        match kind {
            SyntaxTokenKind::Comment => base_style
                .fg(self.theme.comment_color())
                .add_modifier(Modifier::ITALIC),
            SyntaxTokenKind::Keyword => base_style.fg(self.theme.keyword_color()),
            SyntaxTokenKind::StringLiteral => base_style.fg(self.theme.string_color()),
            SyntaxTokenKind::TypeName => base_style.fg(self.theme.type_name_color()),
            SyntaxTokenKind::Function => base_style.fg(self.theme.function_color()),
            SyntaxTokenKind::Constant => base_style.fg(self.theme.constant_color()),
            SyntaxTokenKind::Operator => base_style.fg(self.theme.fg_color()),
            SyntaxTokenKind::Punctuation => base_style.fg(self.theme.comment_color()),
            SyntaxTokenKind::Variable => base_style.fg(self.theme.fg_color()),
            SyntaxTokenKind::Plain => base_style,
        }
    }

    fn highlight_line(
        &self,
        line: &str,
        line_idx: usize,
        in_code_block: bool,
        parse_state: &mut Option<CodeParseState>,
        language: Option<&str>,
    ) -> Line<'a> {
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
            } else if let Some(state) = parse_state {
                // Syntax-highlighted code block content
                if let Some(highlighter) = self.syntax_highlighter {
                    let code_base = base_style.bg(self.theme.panel_color());
                    let tokens = highlighter.tokenize_line(line, state);
                    for token in tokens {
                        let style = self.syntax_token_style(&token.kind, code_base);
                        spans.push(Span::styled(token.text, style));
                    }
                } else {
                    // No highlighter available: plain panel bg
                    spans.push(Span::styled(
                        line.to_string(),
                        base_style.bg(self.theme.panel_color()),
                    ));
                }
            } else if language.is_some() {
                // Language specified but no parse state (shouldn't happen normally)
                spans.push(Span::styled(
                    line.to_string(),
                    base_style.bg(self.theme.panel_color()),
                ));
            } else {
                // Unknown language: plain panel bg
                spans.push(Span::styled(
                    line.to_string(),
                    base_style.bg(self.theme.panel_color()),
                ));
            }
            return Line::from(spans);
        }

        // HTML comment lines: entire line in comment color + italic
        let trimmed = line.trim();
        if trimmed.starts_with("<!--") {
            spans.push(Span::styled(
                line.to_string(),
                base_style
                    .fg(self.theme.comment_color())
                    .add_modifier(Modifier::ITALIC),
            ));
            return Line::from(spans);
        }

        // Line-level patterns (heading, checkbox, smart tag)
        if let Some(m) = HEADING_RE.find(line) {
            let prefix = &line[..m.end()];
            let rest = &line[m.end()..];

            // Heading prefix with keyword + bold
            spans.push(Span::styled(
                prefix.to_string(),
                base_style
                    .fg(self.theme.keyword_color())
                    .add_modifier(Modifier::BOLD),
            ));

            // Rest portion: tokenize inline with keyword + bold as base
            let heading_base = base_style
                .fg(self.theme.keyword_color())
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
        } else if CLOSING_TAG_RE.is_match(line) {
            spans.push(Span::styled(
                line.to_string(),
                base_style
                    .fg(self.theme.error_color())
                    .add_modifier(Modifier::ITALIC),
            ));
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
            .enumerate()
            .take(sr)
            .map(|(i, l)| {
                let hi = self.hanging_indents.get(i).copied().unwrap_or(0);
                wrap_calc::display_rows_for_line(l, inner.width, hi)
            })
            .sum();

        for row in sr..=er {
            let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let hi = self.hanging_indents.get(row).copied().unwrap_or(0);

            let col_start = if row == sr { sc } else { 0 };
            let col_end = if row == er {
                ec + 1
            } else {
                graphemes.len() + 1
            };
            let col_end = col_end.min(graphemes.len() + 1);

            let positions =
                wrap_calc::visual_positions_in_range(line, col_start, col_end, inner.width, hi);

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

            rows_before += wrap_calc::display_rows_for_line(line, inner.width, hi);
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
            .enumerate()
            .take(start_row)
            .map(|(i, l)| {
                let hi = self.hanging_indents.get(i).copied().unwrap_or(0);
                wrap_calc::display_rows_for_line(l, inner.width, hi)
            })
            .sum();

        for row in start_row..=end_row {
            let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
            let hi = self.hanging_indents.get(row).copied().unwrap_or(0);
            let num_display_rows = wrap_calc::display_rows_for_line(line, inner.width, hi);

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
        let w = if inner.width == 0 {
            1
        } else {
            inner.width as usize
        };

        let mut rows_before: u16 = content_lines
            .iter()
            .enumerate()
            .take(top_row)
            .map(|(i, l)| {
                let hi = self.hanging_indents.get(i).copied().unwrap_or(0);
                wrap_calc::display_rows_for_line(l, inner.width, hi)
            })
            .sum();

        for row in top_row..=bottom_row {
            let line = content_lines.get(row).map(|s| s.as_str()).unwrap_or("");
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let hi = self.hanging_indents.get(row).copied().unwrap_or(0);

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

            rows_before += wrap_calc::display_rows_for_line(line, inner.width, hi);
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

        // Pre-compute code block info (with language detection)
        let code_block_info = compute_code_block_info(self.content);

        let default_style = Style::default().bg(self.theme.bg_color());

        // Pre-split styled lines using character-level wrapping so that
        // the rendered text matches wrap_calc's cursor position calculations.
        // Track ParseState across lines within each code block.
        let mut parse_state: Option<CodeParseState> = None;

        let source_lines: Vec<&str> = self.content.lines().collect();
        let mut highlighted_lines: Vec<Line> = Vec::with_capacity(source_lines.len());

        for (idx, line) in source_lines.iter().enumerate() {
            let info = code_block_info.get(idx);
            let in_code_block = info.map(|i| i.in_code_block).unwrap_or(false);
            let language = info.and_then(|i| i.language.as_deref());

            let trimmed = line.trim();

            // Manage parse state transitions
            if in_code_block && trimmed.starts_with("```") {
                if parse_state.is_some() {
                    // Closing fence: render with current state, then reset
                    let styled = self.highlight_line(line, idx, in_code_block, &mut parse_state, language);
                    highlighted_lines.push(styled);
                    parse_state = None;
                    continue;
                } else {
                    // Opening fence: create parse state for known language
                    if let Some(lang) = language {
                        if let Some(highlighter) = self.syntax_highlighter {
                            if let Some(syntax) = highlighter.find_syntax(lang) {
                                parse_state = Some(highlighter.create_parse_state(syntax));
                            }
                        }
                    }
                    let styled = self.highlight_line(line, idx, in_code_block, &mut parse_state, language);
                    highlighted_lines.push(styled);
                    continue;
                }
            }

            let styled = self.highlight_line(line, idx, in_code_block, &mut parse_state, language);
            highlighted_lines.push(styled);
        }

        let display_lines: Vec<Line> = highlighted_lines
            .into_iter()
            .enumerate()
            .flat_map(|(idx, line)| {
                let hi = self.hanging_indents.get(idx).copied().unwrap_or(0);
                split_line_by_width(line, inner.width, hi, default_style)
            })
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
                    .enumerate()
                    .take(match_row)
                    .map(|(i, l)| {
                        let hi = self.hanging_indents.get(i).copied().unwrap_or(0);
                        wrap_calc::display_rows_for_line(l, inner.width, hi)
                    })
                    .sum();

                let line = &content_lines[match_row];
                let hi = self.hanging_indents.get(match_row).copied().unwrap_or(0);
                let positions = wrap_calc::visual_positions_in_range(
                    line,
                    match_col,
                    match_col + match_len,
                    inner.width,
                    hi,
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
                self.hanging_indents,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_info_with_language() {
        let content = "hello\n```rust\nlet x = 1;\n```\nworld";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 5);
        // Line 0: "hello" - not in code block
        assert!(!info[0].in_code_block);
        assert!(info[0].language.is_none());
        // Line 1: "```rust" - opening fence, in code block with language
        assert!(info[1].in_code_block);
        assert_eq!(info[1].language.as_deref(), Some("rust"));
        // Line 2: "let x = 1;" - code content, inherits language
        assert!(info[2].in_code_block);
        assert_eq!(info[2].language.as_deref(), Some("rust"));
        // Line 3: "```" - closing fence
        assert!(info[3].in_code_block);
        // Line 4: "world" - not in code block
        assert!(!info[4].in_code_block);
        assert!(info[4].language.is_none());
    }

    #[test]
    fn test_code_block_info_no_language() {
        let content = "```\ncode\n```";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 3);
        assert!(info[0].in_code_block);
        assert!(info[0].language.is_none());
        assert!(info[1].in_code_block);
        assert!(info[1].language.is_none());
        assert!(info[2].in_code_block);
    }

    #[test]
    fn test_code_block_info_no_code_blocks() {
        let content = "no code blocks here";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 1);
        assert!(!info[0].in_code_block);
        assert!(info[0].language.is_none());
    }

    #[test]
    fn test_code_block_info_empty_content() {
        let content = "";
        let info = compute_code_block_info(content);
        assert!(info.is_empty());
    }

    #[test]
    fn test_code_block_info_multiple_blocks() {
        let content = "text\n```python\nprint('hi')\n```\nmiddle\n```js\nconsole.log('hi');\n```\nend";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 9);

        // First block: python
        assert!(!info[0].in_code_block); // "text"
        assert!(info[1].in_code_block); // ```python
        assert_eq!(info[1].language.as_deref(), Some("python"));
        assert!(info[2].in_code_block); // print('hi')
        assert_eq!(info[2].language.as_deref(), Some("python"));
        assert!(info[3].in_code_block); // ```

        // Between blocks
        assert!(!info[4].in_code_block); // "middle"

        // Second block: js
        assert!(info[5].in_code_block); // ```js
        assert_eq!(info[5].language.as_deref(), Some("js"));
        assert!(info[6].in_code_block); // console.log('hi');
        assert_eq!(info[6].language.as_deref(), Some("js"));
        assert!(info[7].in_code_block); // ```

        // After blocks
        assert!(!info[8].in_code_block); // "end"
    }

    #[test]
    fn test_code_block_info_language_with_extra_text() {
        // e.g. ```rust,no_run or ```python3 some_flag
        let content = "```rust,no_run\ncode\n```";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 3);
        assert!(info[0].in_code_block);
        // First word after ``` should be the language
        // "rust,no_run" is the first word (no space)
        assert!(info[0].language.is_some());
    }

    #[test]
    fn test_code_block_info_unclosed_block() {
        let content = "```rust\nlet x = 1;\nno closing fence";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 3);
        assert!(info[0].in_code_block);
        assert_eq!(info[0].language.as_deref(), Some("rust"));
        assert!(info[1].in_code_block);
        assert!(info[2].in_code_block); // still in code block since no closing fence
    }

    #[test]
    fn test_code_block_info_indented_fence() {
        // Indented code fences (common in lists)
        let content = "  ```python\n  code\n  ```";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 3);
        // The function uses trimmed line, so indented fences should work
        assert!(info[0].in_code_block);
        assert_eq!(info[0].language.as_deref(), Some("python"));
    }

    #[test]
    fn test_code_block_info_only_fence_lines() {
        let content = "```\n```";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 2);
        assert!(info[0].in_code_block);
        assert!(info[0].language.is_none());
        assert!(info[1].in_code_block); // closing fence is also in code block
    }

    #[test]
    fn test_code_block_info_multiline_content() {
        let content = "# Header\n\n```toml\n[package]\nname = \"test\"\nversion = \"0.1\"\n```\n\nMore text";
        let info = compute_code_block_info(content);
        assert_eq!(info.len(), 9);
        assert!(!info[0].in_code_block); // # Header
        assert!(!info[1].in_code_block); // empty line
        assert!(info[2].in_code_block); // ```toml
        assert_eq!(info[2].language.as_deref(), Some("toml"));
        assert!(info[3].in_code_block); // [package]
        assert!(info[4].in_code_block); // name = "test"
        assert!(info[5].in_code_block); // version = "0.1"
        assert!(info[6].in_code_block); // ```
        assert!(!info[7].in_code_block); // empty line
        assert!(!info[8].in_code_block); // More text
    }
}

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::types::Theme;

pub struct ConfirmOverlay<'a> {
    title: &'a str,
    theme: &'a Theme,
}

impl<'a> ConfirmOverlay<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self { title, theme }
    }
}

impl Widget for ConfirmOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_width = 42.min(area.width.saturating_sub(4));
        let overlay_height = 5.min(area.height.saturating_sub(4));

        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        Clear.render(overlay_area, buf);

        let block = Block::default()
            .title(" Confirm Delete ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.warning_color()))
            .style(Style::default().bg(self.theme.panel_color()));

        let inner = block.inner(overlay_area);
        block.render(overlay_area, buf);

        // Truncate title if it would overflow the inner width
        let max_title_len = inner.width.saturating_sub(10) as usize; // "Delete ''?" = 10 chars
        let display_title = if self.title.len() > max_title_len {
            format!("{}...", &self.title[..max_title_len.saturating_sub(3)])
        } else {
            self.title.to_string()
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Delete '", Style::default().fg(self.theme.fg_color())),
                Span::styled(
                    display_title,
                    Style::default()
                        .fg(self.theme.warning_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("'?", Style::default().fg(self.theme.fg_color())),
            ]),
            Line::from(vec![
                Span::styled(
                    "y",
                    Style::default()
                        .fg(self.theme.accent_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Yes  ", Style::default().fg(self.theme.border_color())),
                Span::styled(
                    "n/Esc",
                    Style::default()
                        .fg(self.theme.accent_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": No", Style::default().fg(self.theme.border_color())),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(self.theme.panel_color()));

        paragraph.render(inner, buf);
    }
}

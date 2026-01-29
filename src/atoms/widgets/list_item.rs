use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::types::{Note, Theme};

pub struct ListItemWidget<'a> {
    note: &'a Note,
    theme: &'a Theme,
    is_selected: bool,
    is_highlighted: bool,
    show_archive_icon: bool,
}

impl<'a> ListItemWidget<'a> {
    pub fn new(note: &'a Note, theme: &'a Theme, is_selected: bool) -> Self {
        Self {
            note,
            theme,
            is_selected,
            is_highlighted: false,
            show_archive_icon: false,
        }
    }

    pub fn highlighted(mut self, highlighted: bool) -> Self {
        self.is_highlighted = highlighted;
        self
    }

    pub fn show_archive_icon(mut self, show: bool) -> Self {
        self.show_archive_icon = show;
        self
    }
}

impl Widget for ListItemWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg_color = if self.is_selected {
            self.theme.selection_color()
        } else if self.is_highlighted {
            self.theme.warning_color()
        } else {
            self.theme.bg_color()
        };

        let border_style = if self.is_selected {
            Style::default().fg(self.theme.accent_color())
        } else {
            Style::default().fg(self.theme.border_color())
        };

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(border_style)
            .style(Style::default().bg(bg_color));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

        let icon = if self.show_archive_icon { "@" } else { "#" };

        let selected_indicator = if self.note.selected {
            Span::styled(
                "* ",
                Style::default()
                    .fg(self.theme.warning_color())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("  ")
        };

        let title_line = Line::from(vec![
            selected_indicator,
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(self.theme.accent_color()),
            ),
            Span::styled(
                self.note.title.clone(),
                Style::default()
                    .fg(self.theme.fg_color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        Paragraph::new(title_line).render(chunks[0], buf);

        let preview = self.note.preview(60);
        let date_str = self.note.created_at.format("%Y-%m-%d").to_string();

        let detail_line = Line::from(vec![
            Span::raw("  "),
            Span::styled(preview, Style::default().fg(self.theme.border_color())),
            Span::styled(
                format!(" | {} ", date_str),
                Style::default().fg(self.theme.border_color()),
            ),
        ]);

        Paragraph::new(detail_line).render(chunks[1], buf);
    }
}

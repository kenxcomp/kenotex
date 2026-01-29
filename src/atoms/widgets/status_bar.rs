use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::types::{AppMode, Theme, View};

pub struct StatusBar<'a> {
    mode: AppMode,
    view: View,
    theme: &'a Theme,
    message: &'a str,
    search_query: &'a str,
    file_name: &'a str,
}

impl<'a> StatusBar<'a> {
    pub fn new(mode: AppMode, view: View, theme: &'a Theme) -> Self {
        Self {
            mode,
            view,
            theme,
            message: "",
            search_query: "",
            file_name: "",
        }
    }

    pub fn message(mut self, message: &'a str) -> Self {
        self.message = message;
        self
    }

    pub fn search_query(mut self, query: &'a str) -> Self {
        self.search_query = query;
        self
    }

    pub fn file_name(mut self, name: &'a str) -> Self {
        self.file_name = name;
        self
    }

    fn mode_color(&self) -> ratatui::style::Color {
        match self.mode {
            AppMode::Normal => self.theme.accent_color(),
            AppMode::Insert => self.theme.success_color(),
            AppMode::Visual => self.theme.warning_color(),
            AppMode::Processing => self.theme.error_color(),
            AppMode::Search => self.theme.warning_color(),
        }
    }

    fn view_icon(&self) -> &'static str {
        match self.view {
            View::Editor => "[]",
            View::DraftList => "=",
            View::ArchiveList => "@",
        }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

        let message_line = if self.mode == AppMode::Search {
            Line::from(vec![
                Span::styled("/", Style::default().fg(self.theme.warning_color())),
                Span::styled(
                    self.search_query.to_string(),
                    Style::default().fg(self.theme.fg_color()),
                ),
                Span::styled(
                    "_",
                    Style::default()
                        .fg(self.theme.cursor_color())
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
            ])
        } else {
            Line::from(Span::styled(
                self.message.to_string(),
                Style::default().fg(self.theme.fg_color()),
            ))
        };

        Paragraph::new(message_line)
            .style(Style::default().bg(self.theme.panel_color()))
            .render(chunks[0], buf);

        let mode_span = Span::styled(
            format!(" {} ", self.mode.as_str()),
            Style::default()
                .bg(self.mode_color())
                .fg(self.theme.bg_color())
                .add_modifier(Modifier::BOLD),
        );

        let view_span = Span::styled(
            format!(" {} {} ", self.view_icon(), self.view.as_str()),
            Style::default()
                .bg(self.theme.border_color())
                .fg(self.theme.fg_color()),
        );

        let file_span = if !self.file_name.is_empty() {
            Span::styled(
                format!(" {} ", self.file_name),
                Style::default()
                    .bg(self.theme.selection_color())
                    .fg(self.theme.fg_color()),
            )
        } else {
            Span::raw("")
        };

        let meta_span = Span::styled(
            " utf-8 | markdown | 100% ",
            Style::default()
                .bg(self.theme.panel_color())
                .fg(self.theme.border_color()),
        );

        let icons_span = Span::styled(
            " [v] [c] [n] ",
            Style::default()
                .bg(self.theme.panel_color())
                .fg(self.theme.border_color()),
        );

        let status_line = Line::from(vec![
            mode_span,
            view_span,
            file_span,
            Span::styled(
                " ".repeat(
                    area.width
                        .saturating_sub(
                            self.mode.as_str().len() as u16
                                + self.view.as_str().len() as u16
                                + self.file_name.len() as u16
                                + 30,
                        )
                        .into(),
                ),
                Style::default().bg(self.theme.panel_color()),
            ),
            meta_span,
            icons_span,
        ]);

        Paragraph::new(status_line)
            .style(Style::default().bg(self.theme.panel_color()))
            .render(chunks[1], buf);
    }
}

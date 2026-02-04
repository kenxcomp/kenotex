use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::types::{AppMode, Theme, View};

pub struct HintBar<'a> {
    mode: AppMode,
    view: View,
    theme: &'a Theme,
}

impl<'a> HintBar<'a> {
    pub fn new(mode: AppMode, view: View, theme: &'a Theme) -> Self {
        Self { mode, view, theme }
    }

    fn hints(&self) -> Vec<(&str, &str)> {
        match (self.view, self.mode) {
            (View::Editor, AppMode::Normal) => vec![
                ("Space", "Leader"),
                ("i", "Insert"),
                ("v", "Visual"),
                ("dd", "DelLine"),
                ("yy", "Yank"),
                ("p", "Paste"),
                ("u", "Undo"),
                ("gcc", "Comment"),
                ("/", "Search"),
                ("^Q", "Quit"),
            ],
            (View::Editor, AppMode::Insert) => vec![
                ("Esc", "Normal"),
                ("^G", "ExtEdit"),
            ],
            (View::Editor, AppMode::Visual) => vec![
                ("Esc", "Normal"),
                ("d", "Delete"),
                ("y", "Yank"),
                ("gc", "Comment"),
                ("Space", "Format"),
                ("hjkl", "Move"),
            ],
            (_, AppMode::Search) => vec![
                ("Enter", "Confirm"),
                ("Esc", "Cancel"),
            ],
            (_, AppMode::ConfirmDelete) => vec![
                ("y", "Confirm"),
                ("n/Esc", "Cancel"),
            ],
            (View::DraftList, AppMode::Normal) => vec![
                ("j/k", "Nav"),
                ("Enter", "Open"),
                ("a", "Archive"),
                ("d", "Delete"),
                ("n", "New"),
                ("A", "Archives"),
                ("/", "Search"),
            ],
            (View::ArchiveList, AppMode::Normal) => vec![
                ("j/k", "Nav"),
                ("Enter", "View"),
                ("r", "Restore"),
                ("d", "Delete"),
                ("Esc", "Back"),
            ],
            _ => vec![],
        }
    }
}

impl Widget for HintBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hints = self.hints();
        if hints.is_empty() {
            return;
        }

        let sep_style = Style::default().fg(self.theme.border_color());
        let key_style = Style::default()
            .fg(self.theme.accent_color())
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(self.theme.border_color());

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(" ", sep_style));

        for (i, (key, desc)) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" \u{2502} ", sep_style));
            }
            spans.push(Span::styled(*key, key_style));
            spans.push(Span::styled(format!(" {}", desc), desc_style));
        }

        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(self.theme.panel_color()))
            .render(area, buf);
    }
}

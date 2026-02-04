use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::types::Theme;

const LEADER_COMMANDS: &[(&str, &str)] = &[
    ("s", "Process blocks"),
    ("l", "Draft list"),
    ("nn", "New note"),
    ("q", "Quit"),
    ("h", "Toggle hints"),
    ("d", "Toggle checkbox"),
    ("mc", "Insert checkbox"),
    ("b", "Bold"),
    ("i", "Italic"),
    ("x", "Strikethrough"),
    ("c", "Inline code"),
    ("C", "Code block"),
];

const POPUP_WIDTH: u16 = 24;

pub struct LeaderPopup<'a> {
    theme: &'a Theme,
}

impl<'a> LeaderPopup<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl Widget for LeaderPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_height = LEADER_COMMANDS.len() as u16 + 2; // +2 for borders

        if area.width < POPUP_WIDTH + 1 || area.height < popup_height + 3 {
            return;
        }

        let x = area.width.saturating_sub(POPUP_WIDTH + 1);
        let y = area.height.saturating_sub(popup_height + 3); // 3 = status bar (2) + hint bar (1)

        let popup_area = Rect::new(x, y, POPUP_WIDTH, popup_height);

        Clear.render(popup_area, buf);

        let block = Block::default()
            .title(" Leader ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent_color()))
            .style(Style::default().bg(self.theme.panel_color()));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let key_style = Style::default()
            .fg(self.theme.accent_color())
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(self.theme.fg_color());

        let lines: Vec<Line> = LEADER_COMMANDS
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(format!(" {:<3}", key), key_style),
                    Span::styled(*desc, desc_style),
                ])
            })
            .collect();

        Paragraph::new(lines)
            .style(Style::default().bg(self.theme.panel_color()))
            .render(inner, buf);
    }
}

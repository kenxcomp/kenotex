use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::types::{BlockType, ProcessingStatus, SmartBlock, Theme};

pub struct ProcessingOverlay<'a> {
    blocks: &'a [SmartBlock],
    theme: &'a Theme,
    current_index: usize,
}

impl<'a> ProcessingOverlay<'a> {
    pub fn new(blocks: &'a [SmartBlock], theme: &'a Theme, current_index: usize) -> Self {
        Self {
            blocks,
            theme,
            current_index,
        }
    }

    fn block_type_color(&self, block_type: BlockType) -> ratatui::style::Color {
        match block_type {
            BlockType::Reminder => self.theme.accent_color(),
            BlockType::Calendar => self.theme.error_color(),
            BlockType::Note => self.theme.warning_color(),
        }
    }

    fn block_type_icon(&self, block_type: BlockType) -> &'static str {
        match block_type {
            BlockType::Reminder => "[v]",
            BlockType::Calendar => "[c]",
            BlockType::Note => "[n]",
        }
    }
}

impl Widget for ProcessingOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_width = area.width.min(60);
        let overlay_height = (self.blocks.len() as u16 * 3 + 6).min(area.height - 4);

        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        Clear.render(overlay_area, buf);

        let block = Block::default()
            .title(" Processing Blocks ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent_color()))
            .style(Style::default().bg(self.theme.panel_color()));

        let inner = block.inner(overlay_area);
        block.render(overlay_area, buf);

        let mut constraints: Vec<Constraint> = self
            .blocks
            .iter()
            .map(|_| Constraint::Length(3))
            .collect();
        constraints.push(Constraint::Min(0));

        let chunks = Layout::vertical(constraints).split(inner);

        for (idx, smart_block) in self.blocks.iter().enumerate() {
            if idx >= chunks.len() - 1 {
                break;
            }

            let is_current = idx == self.current_index;

            let border_color = if smart_block.status == ProcessingStatus::Sent {
                self.theme.success_color()
            } else if smart_block.status == ProcessingStatus::Skipped {
                self.theme.border_color()
            } else if is_current {
                self.theme.accent_color()
            } else {
                self.theme.border_color()
            };

            let item_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(self.theme.bg_color()));

            let item_inner = item_block.inner(chunks[idx]);
            item_block.render(chunks[idx], buf);

            let type_icon = self.block_type_icon(smart_block.block_type);
            let type_color = self.block_type_color(smart_block.block_type);

            let status_icon = match smart_block.status {
                ProcessingStatus::Pending => " ",
                ProcessingStatus::Sent => "+",
                ProcessingStatus::Failed => "x",
                ProcessingStatus::Skipped => "-",
            };

            let status_color = match smart_block.status {
                ProcessingStatus::Pending => self.theme.border_color(),
                ProcessingStatus::Sent => self.theme.success_color(),
                ProcessingStatus::Failed => self.theme.error_color(),
                ProcessingStatus::Skipped => self.theme.warning_color(),
            };

            let preview = smart_block.preview(40);

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", type_icon),
                    Style::default().fg(type_color),
                ),
                Span::styled(
                    format!("{}: ", smart_block.block_type.as_str()),
                    Style::default()
                        .fg(type_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(preview, Style::default().fg(self.theme.fg_color())),
            ]);

            Paragraph::new(line)
                .style(Style::default().bg(self.theme.bg_color()))
                .render(item_inner, buf);
        }
    }
}

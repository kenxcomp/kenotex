mod atoms;
mod coordinator;
mod molecules;
mod types;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};

use coordinator::{App, EventDispatcher};
use types::{AppMode, View};

use crate::atoms::storage::{
    cleanup_temp_file, read_temp_file, resolve_editor, spawn_editor, write_temp_file,
};
use crate::atoms::widgets::{EditorWidget, ProcessingOverlay, StatusBar};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_mode = app.mode;

    loop {
        // Update cursor style when mode changes
        if app.mode != last_mode {
            let cursor_style = match app.mode {
                AppMode::Insert => SetCursorStyle::BlinkingBar,
                _ => SetCursorStyle::SteadyBlock,
            };
            execute!(terminal.backend_mut(), cursor_style)?;
            last_mode = app.mode;
        }

        terminal.draw(|f| ui(f, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if app.mode == AppMode::Processing {
                    if key.code == KeyCode::Esc {
                        app.finish_processing();
                    }
                    continue;
                }

                if matches!(app.view, View::DraftList | View::ArchiveList)
                    && app.mode == AppMode::Normal
                    && !app.vim_mode.is_leader_pending()
                {
                    EventDispatcher::handle_list_key(app, key)?;
                }

                EventDispatcher::handle_key(app, key)?;

                if app.external_editor_requested {
                    app.external_editor_requested = false;
                    handle_external_editor(terminal, app)?;
                    continue;
                }
            }
        }

        if app.mode == AppMode::Processing {
            std::thread::sleep(Duration::from_millis(400));
            if !app.process_next_block() {
                app.finish_processing();
            }
        }

        app.auto_save_if_needed()?;

        if app.should_quit {
            if app.dirty {
                app.save_current_note()?;
            }
            break;
        }
    }

    Ok(())
}

fn handle_external_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let editor = resolve_editor();
    let temp_path = write_temp_file(&app.buffer.to_string())?;

    // Suspend TUI
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    // Spawn editor (blocks until exit)
    let editor_ok = spawn_editor(&editor, &temp_path);

    // Restore TUI unconditionally
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?;

    match editor_ok {
        Ok(true) => {
            let content = read_temp_file(&temp_path)?;
            app.apply_external_editor_result(content);
        }
        Ok(false) => {
            app.set_message("External editor exited with error");
        }
        Err(e) => {
            app.set_message(&format!("Failed to launch editor: {}", e));
        }
    }

    cleanup_temp_file(&temp_path);
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let theme = app.theme();

    let bg_style = Style::default().bg(theme.bg_color());
    f.render_widget(Clear, f.area());
    f.render_widget(
        Block::default().style(bg_style),
        f.area(),
    );

    let main_chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(f.area());

    match app.view {
        View::Editor => {
            render_editor(f, app, main_chunks[0]);
        }
        View::DraftList => {
            render_draft_list(f, app, main_chunks[0]);
        }
        View::ArchiveList => {
            render_archive_list(f, app, main_chunks[0]);
        }
    }

    let status_bar = StatusBar::new(app.mode, app.view, theme)
        .message(&app.command_message)
        .search_query(&app.search_query)
        .file_name(
            app.current_note
                .as_ref()
                .map(|n| n.title.as_str())
                .unwrap_or(""),
        );
    f.render_widget(status_bar, main_chunks[1]);

    if app.mode == AppMode::Processing && !app.processing_blocks.is_empty() {
        let overlay = ProcessingOverlay::new(&app.processing_blocks, theme, app.processing_index);
        f.render_widget(overlay, f.area());
    }
}

fn render_editor(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let title = app
        .current_note
        .as_ref()
        .map(|n| n.title.as_str())
        .unwrap_or("Untitled");

    let content = app.buffer.to_string();
    let editor = EditorWidget::new(
        &content,
        app.buffer.cursor_position(),
        theme,
        app.mode,
        title,
    )
    .scroll_offset(app.scroll_offset());

    f.render_widget(editor, area);

    // In Insert mode, show native terminal cursor (I-beam)
    if app.mode == AppMode::Insert {
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;

        let (cursor_row, cursor_col) = app.buffer.cursor_position();
        let inner_x = area.x + 1; // Account for border
        let inner_y = area.y + 1;

        // Calculate display width for cursor position
        let display_offset: u16 = content
            .lines()
            .nth(cursor_row)
            .map(|line| {
                line.graphemes(true)
                    .take(cursor_col)
                    .map(|g| g.width())
                    .sum::<usize>() as u16
            })
            .unwrap_or(0);

        let cursor_x = inner_x + display_offset;
        let cursor_y = inner_y + cursor_row as u16 - app.scroll_offset();

        // Set cursor position for native terminal cursor
        if cursor_y >= inner_y && cursor_y < area.y + area.height - 1 {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_draft_list(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header_chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Drafts ",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("({} items)", app.draft_list.total_count()),
            Style::default().fg(theme.border_color()),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color()))
            .style(Style::default().bg(theme.bg_color())),
    );
    f.render_widget(header, header_chunks[0]);

    let notes = app.draft_list.filtered_notes();
    let selected_idx = app.draft_list.selected_index();

    if notes.is_empty() {
        let empty = Paragraph::new("No drafts. Press 'n' to create one.")
            .style(Style::default().fg(theme.border_color()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_color())),
            );
        f.render_widget(empty, header_chunks[1]);
    } else {
        let items: Vec<ListItem> = notes
            .iter()
            .enumerate()
            .map(|(idx, note)| {
                let is_selected = idx == selected_idx;
                let style = if is_selected {
                    Style::default()
                        .bg(theme.selection_color())
                        .fg(theme.fg_color())
                } else {
                    Style::default().fg(theme.fg_color())
                };

                let prefix = if is_selected { "> " } else { "  " };
                let selected_marker = if note.selected { "* " } else { "" };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(
                        selected_marker,
                        Style::default().fg(theme.warning_color()),
                    ),
                    Span::styled(&note.title, style.add_modifier(Modifier::BOLD)),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .style(Style::default().bg(theme.bg_color())),
        );
        f.render_widget(list, header_chunks[1]);
    }

    let help = Paragraph::new("[j/k] Navigate  [Enter] Edit  [a] Archive  [d] Delete  [n] New  [A] Archives  [/] Search")
        .style(Style::default().fg(theme.border_color()));
    let help_area = Rect::new(area.x + 1, area.y + area.height - 1, area.width - 2, 1);
    f.render_widget(help, help_area);
}

fn render_archive_list(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header_chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Archive ",
            Style::default()
                .fg(theme.warning_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("({} items)", app.archive_list.len()),
            Style::default().fg(theme.border_color()),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color()))
            .style(Style::default().bg(theme.bg_color())),
    );
    f.render_widget(header, header_chunks[0]);

    let notes = app.archive_list.filtered_notes();
    let selected_idx = app.archive_list.selected_index();

    if notes.is_empty() {
        let empty = Paragraph::new("No archived notes.")
            .style(Style::default().fg(theme.border_color()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_color())),
            );
        f.render_widget(empty, header_chunks[1]);
    } else {
        let items: Vec<ListItem> = notes
            .iter()
            .enumerate()
            .map(|(idx, note)| {
                let is_selected = idx == selected_idx;
                let style = if is_selected {
                    Style::default()
                        .bg(theme.selection_color())
                        .fg(theme.fg_color())
                } else {
                    Style::default().fg(theme.fg_color())
                };

                let prefix = if is_selected { "> " } else { "  " };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled("@ ", Style::default().fg(theme.warning_color())),
                    Span::styled(&note.title, style.add_modifier(Modifier::BOLD)),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .style(Style::default().bg(theme.bg_color())),
        );
        f.render_widget(list, header_chunks[1]);
    }

    let help = Paragraph::new("[j/k] Navigate  [Enter] View  [r] Restore  [d] Delete  [Esc] Back")
        .style(Style::default().fg(theme.border_color()));
    let help_area = Rect::new(area.x + 1, area.y + area.height - 1, area.width - 2, 1);
    f.render_widget(help, help_area);
}

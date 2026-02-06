mod atoms;
mod coordinator;
mod molecules;
mod types;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use coordinator::{App, EventDispatcher};
use types::{AppMode, View};

use crate::atoms::storage::file_watcher::{self, FileWatcherHandle};
use crate::atoms::storage::{
    cleanup_temp_file, read_temp_file, resolve_editor, spawn_editor, write_temp_file,
};
use crate::atoms::widgets::{
    ConfirmOverlay, EditorWidget, HintBar, LeaderPopup, ProcessingOverlay, StatusBar,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("kenotex {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    let watcher_handle = if app.config.general.file_watch {
        let drafts = app.data_dir.join("drafts");
        let archives = app.data_dir.join("archives");
        match file_watcher::start_watcher(
            &drafts,
            &archives,
            app.config.general.file_watch_debounce_ms,
        ) {
            Ok(handle) => Some(handle),
            Err(e) => {
                app.set_message(&format!("File watcher failed: {}", e));
                None
            }
        }
    } else {
        None
    };

    let result = run_app(&mut terminal, &mut app, watcher_handle.as_ref());

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    watcher: Option<&FileWatcherHandle>,
) -> Result<()> {
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
            match event::read()? {
                Event::Key(key) => {
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
                        if EventDispatcher::handle_list_key(app, key)? {
                            continue;
                        }
                    }

                    EventDispatcher::handle_key(app, key)?;

                    if app.external_editor_requested {
                        app.external_editor_requested = false;
                        handle_external_editor(terminal, app)?;
                        continue;
                    }
                }
                Event::Paste(text) => {
                    EventDispatcher::handle_paste(app, text)?;
                }
                _ => {}
            }
        }

        // Process file watcher events (non-blocking)
        if let Some(watcher) = watcher {
            while let Ok(event) = watcher.receiver.try_recv() {
                if let Err(e) = app.handle_file_event(event) {
                    app.set_message(&format!("File event error: {}", e));
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
        DisableMouseCapture,
        DisableBracketedPaste
    )?;

    // Spawn editor (blocks until exit)
    let editor_ok = spawn_editor(&editor, &temp_path);

    // Restore TUI unconditionally
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
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
    f.render_widget(Block::default().style(bg_style), f.area());

    let hint_height = if app.show_hints { 1 } else { 0 };
    let main_chunks = Layout::vertical([
        Constraint::Min(1),              // [0] content
        Constraint::Length(hint_height), // [1] hint bar
        Constraint::Length(2),           // [2] status bar
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

    if app.show_hints {
        f.render_widget(HintBar::new(app.mode, app.view, theme), main_chunks[1]);
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
    f.render_widget(status_bar, main_chunks[2]);

    if app.vim_mode.is_leader_pending() {
        f.render_widget(LeaderPopup::new(theme), f.area());
    }

    if app.mode == AppMode::Processing && !app.processing_blocks.is_empty() {
        let overlay = ProcessingOverlay::new(&app.processing_blocks, theme, app.processing_index);
        f.render_widget(overlay, f.area());
    }

    if app.mode == AppMode::ConfirmDelete
        && let Some(title) = &app.pending_delete_title
    {
        f.render_widget(ConfirmOverlay::new(title, theme), f.area());
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
    let search_matches = app.buffer.find_all(&app.search_query);
    let editor = EditorWidget::new(
        &content,
        app.buffer.cursor_position(),
        theme,
        app.mode,
        title,
    )
    .scroll_offset(app.scroll_offset(area.width, area.height))
    .visual_selection(app.get_visual_selection())
    .search_matches(&search_matches);

    f.render_widget(editor, area);

    // In Insert mode, show native terminal cursor (I-beam)
    if app.mode == AppMode::Insert {
        use crate::atoms::widgets::wrap_calc;

        let (cursor_row, cursor_col) = app.buffer.cursor_position();
        let inner_x = area.x + 1; // Account for border
        let inner_y = area.y + 1;
        let inner_width = area.width.saturating_sub(2);

        let content_lines: Vec<String> = content.lines().map(String::from).collect();
        let vpos =
            wrap_calc::visual_cursor_position(&content_lines, cursor_row, cursor_col, inner_width);

        let cursor_x = inner_x + vpos.col;
        let scroll = app.scroll_offset(area.width, area.height);
        let cursor_y = inner_y + vpos.rows_before + vpos.wrap_row - scroll;

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
                    Span::styled(selected_marker, Style::default().fg(theme.warning_color())),
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
}

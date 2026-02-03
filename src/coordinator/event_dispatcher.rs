use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::App;
use crate::atoms::storage::{clipboard_copy, clipboard_paste};
use crate::molecules::editor::list_prefix;
use crate::molecules::editor::VimAction;
use crate::types::{AppMode, View};

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
        let action = app.vim_mode.handle_key(key, app.mode);

        match app.mode {
            AppMode::Normal => Self::handle_normal_action(app, action)?,
            AppMode::Insert => Self::handle_insert_action(app, action)?,
            AppMode::Visual => Self::handle_visual_action(app, action)?,
            AppMode::Search => Self::handle_search_action(app, action, key)?,
            AppMode::Processing => {}
        }

        Ok(())
    }

    fn handle_normal_action(app: &mut App, action: VimAction) -> Result<()> {
        match app.view {
            View::Editor => Self::handle_editor_normal(app, action)?,
            View::DraftList | View::ArchiveList => Self::handle_list_normal(app, action)?,
        }
        Ok(())
    }

    fn handle_editor_normal(app: &mut App, action: VimAction) -> Result<()> {
        match action {
            VimAction::MoveLeft => app.buffer.move_left(),
            VimAction::MoveRight => app.buffer.move_right(),
            VimAction::MoveUp => app.buffer.move_up(),
            VimAction::MoveDown => app.buffer.move_down(),
            VimAction::MoveWordForward => app.buffer.move_word_forward(),
            VimAction::MoveWordBackward => app.buffer.move_word_backward(),
            VimAction::MoveLineStart => app.buffer.move_to_line_start(),
            VimAction::MoveLineEnd => app.buffer.move_to_line_end(),
            VimAction::MoveFileStart => app.buffer.move_to_first_line(),
            VimAction::MoveFileEnd => app.buffer.move_to_last_line(),

            VimAction::InsertMode => {
                app.buffer.save_undo_snapshot();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeAppend => {
                app.buffer.save_undo_snapshot();
                app.buffer.move_right();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeLineEnd => {
                app.buffer.save_undo_snapshot();
                app.buffer.move_to_line_end();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeLineStart => {
                app.buffer.save_undo_snapshot();
                app.buffer.move_to_line_start();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertLineBelow => {
                app.buffer.save_undo_snapshot();
                let line = app.buffer.current_line_content().to_string();
                if list_prefix::is_prefix_only(&line) {
                    app.buffer.clear_current_line();
                    app.buffer.insert_line_below();
                } else if let Some(prefix) = list_prefix::detect_list_prefix(&line) {
                    let full_prefix = format!("{}{}", prefix.indent, prefix.continuation);
                    app.buffer.insert_line_below_with_prefix(&full_prefix);
                } else {
                    app.buffer.insert_line_below();
                }
                app.set_mode(AppMode::Insert);
                app.dirty = true;
                app.set_message("-- INSERT --");
            }
            VimAction::InsertLineAbove => {
                app.buffer.save_undo_snapshot();
                app.buffer.insert_line_above();
                app.set_mode(AppMode::Insert);
                app.dirty = true;
                app.set_message("-- INSERT --");
            }

            VimAction::DeleteChar => {
                app.buffer.save_undo_snapshot();
                app.buffer.delete_char();
                app.dirty = true;
            }
            VimAction::DeleteLine => {
                app.buffer.save_undo_snapshot();
                app.buffer.delete_line();
                app.dirty = true;
            }

            VimAction::Delete(motion) => {
                app.buffer.save_undo_snapshot();
                let (text, linewise) = app.buffer.apply_motion_delete(motion);
                let _ = clipboard_copy(&text);
                app.last_yank_linewise = linewise;
                app.dirty = true;
            }
            VimAction::Yank(motion) => {
                let (text, linewise) = app.buffer.apply_motion_yank(motion);
                let _ = clipboard_copy(&text);
                app.last_yank_linewise = linewise;
                app.set_message("Yanked");
            }
            VimAction::PasteAfter => {
                if let Ok(text) = clipboard_paste() {
                    if !text.is_empty() {
                        app.buffer.save_undo_snapshot();
                        if app.last_yank_linewise {
                            app.buffer.paste_line_below(&text);
                        } else {
                            app.buffer.paste_after_cursor(&text);
                        }
                        app.dirty = true;
                    }
                }
            }
            VimAction::PasteBefore => {
                if let Ok(text) = clipboard_paste() {
                    if !text.is_empty() {
                        app.buffer.save_undo_snapshot();
                        if app.last_yank_linewise {
                            app.buffer.paste_line_above(&text);
                        } else {
                            app.buffer.paste_before_cursor(&text);
                        }
                        app.dirty = true;
                    }
                }
            }

            VimAction::Undo => {
                if app.buffer.undo() {
                    app.dirty = true;
                    app.set_message("Undo");
                } else {
                    app.set_message("Already at oldest change");
                }
            }
            VimAction::Redo => {
                if app.buffer.redo() {
                    app.dirty = true;
                    app.set_message("Redo");
                } else {
                    app.set_message("Already at newest change");
                }
            }

            VimAction::EnterVisualMode => {
                app.visual_anchor = Some(app.buffer.cursor_position());
                app.set_mode(AppMode::Visual);
                app.set_message("-- VISUAL --");
            }

            VimAction::LeaderKey => {
                app.set_message("LEADER");
            }
            VimAction::LeaderList => {
                if app.dirty {
                    app.save_current_note()?;
                }
                app.set_view(View::DraftList);
                app.set_message("");
            }
            VimAction::LeaderNew => {
                app.new_note();
            }
            VimAction::LeaderProcess => {
                app.start_processing();
            }

            VimAction::InsertCheckbox => {
                app.buffer.save_undo_snapshot();
                app.buffer.insert_checkbox();
                app.dirty = true;
            }
            VimAction::ToggleCheckbox => {
                app.buffer.save_undo_snapshot();
                app.buffer.toggle_checkbox();
                app.dirty = true;
            }

            VimAction::ToggleHints => {
                app.toggle_hints();
            }
            VimAction::CycleTheme => {
                app.cycle_theme();
            }
            VimAction::Search => {
                app.set_mode(AppMode::Search);
            }
            VimAction::ReloadBuffer => {
                app.reload_current_note_from_disk()?;
                app.set_message("File reloaded");
            }
            VimAction::ExternalEditor => {
                app.request_external_editor();
            }
            VimAction::Quit => {
                app.should_quit = true;
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_list_normal(app: &mut App, action: VimAction) -> Result<()> {
        match action {
            VimAction::MoveUp => {
                if app.view == View::DraftList {
                    app.draft_list.move_up();
                } else {
                    app.archive_list.move_up();
                }
            }
            VimAction::MoveDown => {
                if app.view == View::DraftList {
                    app.draft_list.move_down();
                } else {
                    app.archive_list.move_down();
                }
            }

            VimAction::InsertMode | VimAction::MoveRight => {
                app.open_selected_note();
            }

            VimAction::LeaderKey => {
                app.set_message("LEADER");
            }
            VimAction::LeaderNew => {
                app.new_note();
            }

            VimAction::Search => {
                app.set_mode(AppMode::Search);
            }

            VimAction::ExitToNormal => {
                if app.view == View::ArchiveList {
                    app.set_view(View::DraftList);
                } else {
                    app.set_view(View::Editor);
                }
            }

            VimAction::ToggleHints => {
                app.toggle_hints();
            }

            VimAction::CycleTheme => {
                app.cycle_theme();
            }

            VimAction::Quit => {
                app.should_quit = true;
            }

            _ => {}
        }

        Ok(())
    }

    pub fn handle_list_key(app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('d') => {
                app.delete_selected_note()?;
                Ok(true)
            }
            KeyCode::Char('a') if app.view == View::DraftList => {
                app.archive_selected_note()?;
                Ok(true)
            }
            KeyCode::Char('r') if app.view == View::ArchiveList => {
                app.restore_selected_note()?;
                Ok(true)
            }
            KeyCode::Char('A') => {
                if app.view == View::DraftList {
                    app.set_view(View::ArchiveList);
                } else {
                    app.set_view(View::DraftList);
                }
                Ok(true)
            }
            KeyCode::Char('n') => {
                app.new_note();
                Ok(true)
            }
            KeyCode::Char(' ') => {
                if app.view == View::DraftList {
                    app.draft_list.toggle_selected();
                }
                Ok(true)
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char('i') => {
                app.open_selected_note();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_insert_action(app: &mut App, action: VimAction) -> Result<()> {
        match action {
            VimAction::InsertChar(c) => {
                app.buffer.insert_char(c);
                app.dirty = true;
            }
            VimAction::InsertNewline => {
                let line = app.buffer.current_line_content().to_string();
                if list_prefix::is_prefix_only(&line) {
                    app.buffer.clear_current_line();
                    app.buffer.insert_newline();
                } else if let Some(prefix) = list_prefix::detect_list_prefix(&line) {
                    let full_prefix = format!("{}{}", prefix.indent, prefix.continuation);
                    app.buffer.insert_newline_with_prefix(&full_prefix);
                } else {
                    app.buffer.insert_newline();
                }
                app.dirty = true;
            }
            VimAction::Backspace => {
                app.buffer.backspace();
                app.dirty = true;
            }
            VimAction::DeleteChar => {
                app.buffer.delete_char();
                app.dirty = true;
            }
            VimAction::MoveLeft => app.buffer.move_left(),
            VimAction::MoveRight => app.buffer.move_right(),
            VimAction::MoveUp => app.buffer.move_up(),
            VimAction::MoveDown => app.buffer.move_down(),
            VimAction::MoveLineStart => app.buffer.move_to_line_start(),
            VimAction::MoveLineEnd => app.buffer.move_to_line_end(),
            VimAction::ExitToNormal => {
                app.set_mode(AppMode::Normal);
                app.clear_message();
            }
            VimAction::ExternalEditor => {
                app.request_external_editor();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_visual_action(app: &mut App, action: VimAction) -> Result<()> {
        match action {
            VimAction::MoveLeft => app.buffer.move_left(),
            VimAction::MoveRight => app.buffer.move_right(),
            VimAction::MoveUp => app.buffer.move_up(),
            VimAction::MoveDown => app.buffer.move_down(),
            VimAction::MoveWordForward => app.buffer.move_word_forward(),
            VimAction::MoveWordBackward => app.buffer.move_word_backward(),
            VimAction::MoveLineStart => app.buffer.move_to_line_start(),
            VimAction::MoveLineEnd => app.buffer.move_to_line_end(),
            VimAction::MoveFileStart => app.buffer.move_to_first_line(),
            VimAction::MoveFileEnd => app.buffer.move_to_last_line(),

            VimAction::VisualDelete => {
                if let Some(((sr, sc), (er, ec))) = app.visual_selection() {
                    app.buffer.save_undo_snapshot();
                    // Include the end character in the deletion
                    let delete_ec = if er < app.buffer.line_count() {
                        let line = app.buffer.content().get(er).map(|l| l.len()).unwrap_or(0);
                        (ec + 1).min(
                            app.buffer
                                .content()
                                .get(er)
                                .map(|l| {
                                    use unicode_segmentation::UnicodeSegmentation;
                                    l.graphemes(true).count()
                                })
                                .unwrap_or(line),
                        )
                    } else {
                        ec + 1
                    };
                    let text = app.buffer.delete_range(sr, sc, er, delete_ec);
                    let _ = clipboard_copy(&text);
                    app.last_yank_linewise = false;
                    app.dirty = true;
                }
                app.visual_anchor = None;
                app.set_mode(AppMode::Normal);
                app.clear_message();
            }
            VimAction::VisualYank => {
                if let Some(((sr, sc), (er, ec))) = app.visual_selection() {
                    let extract_ec = if er < app.buffer.line_count() {
                        (ec + 1).min(
                            app.buffer
                                .content()
                                .get(er)
                                .map(|l| {
                                    use unicode_segmentation::UnicodeSegmentation;
                                    l.graphemes(true).count()
                                })
                                .unwrap_or(ec + 1),
                        )
                    } else {
                        ec + 1
                    };
                    let text = app.buffer.extract_range(sr, sc, er, extract_ec);
                    let _ = clipboard_copy(&text);
                    app.last_yank_linewise = false;
                    app.set_message("Yanked");
                }
                app.visual_anchor = None;
                app.set_mode(AppMode::Normal);
            }

            VimAction::ExitToNormal => {
                app.visual_anchor = None;
                app.set_mode(AppMode::Normal);
                app.clear_message();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_action(app: &mut App, action: VimAction, key: KeyEvent) -> Result<()> {
        match action {
            VimAction::InsertChar(c) => {
                app.search_query.push(c);
                match app.view {
                    View::DraftList => app.draft_list.add_search_char(c),
                    View::ArchiveList => app.archive_list.add_search_char(c),
                    View::Editor => {}
                }
            }
            VimAction::Backspace => {
                app.search_query.pop();
                match app.view {
                    View::DraftList => app.draft_list.remove_search_char(),
                    View::ArchiveList => app.archive_list.remove_search_char(),
                    View::Editor => {}
                }
            }
            VimAction::ExitToNormal => {
                app.set_mode(AppMode::Normal);
                if key.code == KeyCode::Esc {
                    app.search_query.clear();
                    match app.view {
                        View::DraftList => app.draft_list.clear_search(),
                        View::ArchiveList => app.archive_list.clear_search(),
                        View::Editor => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

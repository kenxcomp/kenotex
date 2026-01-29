use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::App;
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
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeAppend => {
                app.buffer.move_right();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeLineEnd => {
                app.buffer.move_to_line_end();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertModeLineStart => {
                app.buffer.move_to_line_start();
                app.set_mode(AppMode::Insert);
                app.set_message("-- INSERT --");
            }
            VimAction::InsertLineBelow => {
                app.buffer.insert_line_below();
                app.set_mode(AppMode::Insert);
                app.dirty = true;
                app.set_message("-- INSERT --");
            }
            VimAction::InsertLineAbove => {
                app.buffer.insert_line_above();
                app.set_mode(AppMode::Insert);
                app.dirty = true;
                app.set_message("-- INSERT --");
            }

            VimAction::DeleteChar => {
                app.buffer.delete_char();
                app.dirty = true;
            }
            VimAction::DeleteLine => {
                app.buffer.delete_line();
                app.dirty = true;
            }

            VimAction::EnterVisualMode => {
                app.set_mode(AppMode::Visual);
                app.set_message("-- VISUAL --");
            }

            VimAction::LeaderKey => {
                app.set_message("LEADER");
            }
            VimAction::LeaderSave => {
                app.save_current_note()?;
            }
            VimAction::LeaderList => {
                app.set_view(View::DraftList);
                app.set_message("");
            }
            VimAction::LeaderNew => {
                app.new_note();
            }
            VimAction::LeaderProcess => {
                app.start_processing();
            }

            VimAction::CycleTheme => {
                app.cycle_theme();
            }
            VimAction::Search => {
                app.set_mode(AppMode::Search);
            }
            VimAction::ExternalEditor => {
                app.set_message("External editor not yet implemented");
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

            VimAction::CycleTheme => {
                app.cycle_theme();
            }

            VimAction::Quit => {
                app.should_quit = true;
            }

            VimAction::DeleteLine => {
                app.delete_selected_note()?;
            }

            _ => {}
        }

        if let Some(KeyCode::Char('a')) = None::<KeyCode> {
        }

        Ok(())
    }

    pub fn handle_list_key(app: &mut App, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('a') if app.view == View::DraftList => {
                app.archive_selected_note()?;
            }
            KeyCode::Char('r') if app.view == View::ArchiveList => {
                app.restore_selected_note()?;
            }
            KeyCode::Char('A') => {
                if app.view == View::DraftList {
                    app.set_view(View::ArchiveList);
                } else {
                    app.set_view(View::DraftList);
                }
            }
            KeyCode::Char('n') => {
                app.new_note();
            }
            KeyCode::Char(' ') => {
                if app.view == View::DraftList {
                    app.draft_list.toggle_selected();
                }
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char('i') => {
                app.open_selected_note();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_action(app: &mut App, action: VimAction) -> Result<()> {
        match action {
            VimAction::InsertChar(c) => {
                app.buffer.insert_char(c);
                app.dirty = true;
            }
            VimAction::InsertNewline => {
                app.buffer.insert_newline();
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
                app.set_message("External editor not yet implemented");
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
            VimAction::ExitToNormal => {
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

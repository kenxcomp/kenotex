use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::App;
use crate::atoms::storage::{clipboard_copy, clipboard_paste};
use crate::molecules::editor::VimAction;
use crate::molecules::editor::list_prefix;
use crate::types::{AppMode, View};

pub struct EventDispatcher;

impl EventDispatcher {
    /// Handle a bracketed paste event (Cmd+V / terminal paste).
    pub fn handle_paste(app: &mut App, text: String) -> Result<()> {
        match app.mode {
            AppMode::Insert => {
                app.buffer.save_undo_snapshot();
                app.buffer.insert_text(&text);
                app.dirty = true;
            }
            AppMode::Normal => {
                if !text.is_empty() {
                    app.buffer.save_undo_snapshot();
                    app.buffer.paste_after_cursor(&text);
                    app.dirty = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if app.mode == AppMode::ConfirmDelete {
            match key.code {
                KeyCode::Char('y') => app.confirm_delete()?,
                KeyCode::Char('n') | KeyCode::Esc => app.cancel_delete(),
                _ => {}
            }
            return Ok(());
        }

        let action = app.vim_mode.handle_key(key, app.mode);

        match app.mode {
            AppMode::Normal => Self::handle_normal_action(app, action)?,
            AppMode::Insert => Self::handle_insert_action(app, action)?,
            AppMode::Visual(_) => Self::handle_visual_action(app, action)?,
            AppMode::Search => Self::handle_search_action(app, action, key)?,
            AppMode::Processing | AppMode::ConfirmDelete => {}
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

            VimAction::Indent => {
                app.buffer.save_undo_snapshot();
                let tab_width = app.config.general.tab_width;
                app.buffer.indent_line(tab_width);
                app.dirty = true;
            }
            VimAction::Dedent => {
                app.buffer.save_undo_snapshot();
                let tab_width = app.config.general.tab_width;
                app.buffer.dedent_line(tab_width);
                app.dirty = true;
            }

            VimAction::EnterVisualMode => {
                // Legacy action - treat as character mode
                app.enter_visual_mode(crate::molecules::editor::VisualType::Character);
                app.set_message("-- VISUAL --");
            }

            VimAction::EnterVisualCharacter => {
                app.enter_visual_mode(crate::molecules::editor::VisualType::Character);
                app.set_message("-- VISUAL --");
            }

            VimAction::EnterVisualLine => {
                app.enter_visual_mode(crate::molecules::editor::VisualType::Line);
                app.set_message("-- VISUAL LINE --");
            }

            VimAction::EnterVisualBlock => {
                app.enter_visual_mode(crate::molecules::editor::VisualType::Block);
                app.set_message("-- VISUAL BLOCK --");
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
            VimAction::ToggleComment => {
                app.buffer.save_undo_snapshot();
                app.buffer.toggle_comment();
                app.dirty = true;
            }
            VimAction::ToggleFormat(f) => {
                app.buffer.save_undo_snapshot();
                app.buffer.toggle_format(f);
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
            VimAction::SearchNext => {
                if !app.search_query.is_empty() {
                    let (row, col) = app.buffer.cursor_position();
                    if let Some((r, c)) = app.buffer.find_next(&app.search_query, row, col) {
                        app.buffer.set_cursor(r, c);
                        app.set_message(&format!("/{}", app.search_query));
                    } else {
                        app.set_message(&format!("Pattern not found: {}", app.search_query));
                    }
                }
            }
            VimAction::SearchPrev => {
                if !app.search_query.is_empty() {
                    let (row, col) = app.buffer.cursor_position();
                    if let Some((r, c)) = app.buffer.find_prev(&app.search_query, row, col) {
                        app.buffer.set_cursor(r, c);
                        app.set_message(&format!("?{}", app.search_query));
                    } else {
                        app.set_message(&format!("Pattern not found: {}", app.search_query));
                    }
                }
            }
            VimAction::ClearSearch | VimAction::ExitToNormal => {
                if !app.search_query.is_empty() {
                    app.search_query.clear();
                    app.clear_message();
                }
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
            VimAction::SearchNext | VimAction::SearchPrev => {
                // List views use filter-based search, n/N are no-ops here
            }
            VimAction::ClearSearch => {
                app.search_query.clear();
                if app.view == View::DraftList {
                    app.draft_list.clear_search();
                } else {
                    app.archive_list.clear_search();
                }
                app.clear_message();
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
                app.request_delete();
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
            KeyCode::Char('n') if app.search_query.is_empty() => {
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
            VimAction::InsertTab => {
                let tab_width = app.config.general.tab_width;
                app.buffer.insert_tab(tab_width);
                app.dirty = true;
            }
            VimAction::Dedent => {
                let tab_width = app.config.general.tab_width;
                app.buffer.dedent_line(tab_width);
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
                app.exit_insert_mode();
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
        use crate::molecules::editor::VisualType;

        let is_block_mode = matches!(app.mode, crate::types::AppMode::Visual(VisualType::Block));

        match action {
            // Movement actions with display-aware handling for Visual Block mode
            VimAction::MoveLeft => {
                if is_block_mode {
                    let (row, col) = app.buffer.cursor_position();
                    let current_display_col = app.buffer.display_col_at(row, col);
                    let target = current_display_col.saturating_sub(1);
                    app.visual_target_display_col = Some(target);

                    let new_col = app.buffer.grapheme_at_display_col(row, target);
                    app.buffer.set_cursor(row, new_col);
                } else {
                    app.buffer.move_left();
                }
            }
            VimAction::MoveRight => {
                if is_block_mode {
                    let (row, col) = app.buffer.cursor_position();
                    let current_display_col = app.buffer.display_col_at(row, col);
                    let target = current_display_col + 1;
                    app.visual_target_display_col = Some(target);

                    let new_col = app.buffer.grapheme_at_display_col(row, target);
                    app.buffer.set_cursor(row, new_col);
                } else {
                    app.buffer.move_right();
                }
            }
            VimAction::MoveUp => {
                if is_block_mode {
                    let (row, col) = app.buffer.cursor_position();
                    let target_col = app
                        .visual_target_display_col
                        .unwrap_or_else(|| app.buffer.display_col_at(row, col));

                    app.buffer.move_up();
                    let new_row = app.buffer.cursor_position().0;
                    let new_col = app.buffer.grapheme_at_display_col(new_row, target_col);
                    app.buffer.set_cursor(new_row, new_col);
                    app.visual_target_display_col = Some(target_col);
                } else {
                    app.buffer.move_up();
                }
            }
            VimAction::MoveDown => {
                if is_block_mode {
                    let (row, col) = app.buffer.cursor_position();
                    let target_col = app
                        .visual_target_display_col
                        .unwrap_or_else(|| app.buffer.display_col_at(row, col));

                    app.buffer.move_down();
                    let new_row = app.buffer.cursor_position().0;
                    let new_col = app.buffer.grapheme_at_display_col(new_row, target_col);
                    app.buffer.set_cursor(new_row, new_col);
                    app.visual_target_display_col = Some(target_col);
                } else {
                    app.buffer.move_down();
                }
            }
            VimAction::MoveWordForward => app.buffer.move_word_forward(),
            VimAction::MoveWordBackward => app.buffer.move_word_backward(),
            VimAction::MoveLineStart => app.buffer.move_to_line_start(),
            VimAction::MoveLineEnd => app.buffer.move_to_line_end(),
            VimAction::MoveFileStart => app.buffer.move_to_first_line(),
            VimAction::MoveFileEnd => app.buffer.move_to_last_line(),

            // Mode switching
            VimAction::SwitchToVisualCharacter => {
                app.switch_visual_type(crate::molecules::editor::VisualType::Character);
                app.set_message("-- VISUAL --");
            }
            VimAction::SwitchToVisualLine => {
                app.switch_visual_type(crate::molecules::editor::VisualType::Line);
                app.set_message("-- VISUAL LINE --");
            }
            VimAction::SwitchToVisualBlock => {
                app.switch_visual_type(crate::molecules::editor::VisualType::Block);
                app.set_message("-- VISUAL BLOCK --");
            }

            // Visual operations
            VimAction::VisualDelete => {
                app.buffer.save_undo_snapshot();
                if let Some(deleted) = app.visual_delete() {
                    let _ = clipboard_copy(&deleted);
                    app.last_yank_linewise = false;
                }
                app.clear_message();
            }

            VimAction::VisualYank => {
                if let Some(yanked) = app.visual_yank() {
                    let _ = clipboard_copy(&yanked);
                    app.last_yank_linewise = false;
                    app.set_message("Yanked");
                }
                app.exit_visual_mode();
            }

            VimAction::VisualIndent => {
                app.buffer.save_undo_snapshot();
                app.visual_indent();
                app.clear_message();
            }

            VimAction::VisualDedent => {
                app.buffer.save_undo_snapshot();
                app.visual_dedent();
                app.clear_message();
            }

            VimAction::VisualToggleComment => {
                app.buffer.save_undo_snapshot();
                app.visual_toggle_comment();
                app.clear_message();
            }

            VimAction::VisualToggleFormat(f) => {
                // For formatting, we need character-wise coordinates
                if let Some(render_selection) = app.get_visual_selection() {
                    if let crate::molecules::editor::RenderSelection::CharacterRange {
                        start,
                        end,
                    } = render_selection
                    {
                        app.buffer.save_undo_snapshot();
                        app.buffer
                            .toggle_format_visual(start.0, start.1, end.0, end.1, f);
                        app.dirty = true;
                    }
                }
                app.exit_visual_mode();
                app.clear_message();
            }

            // Block-specific operations
            VimAction::VisualBlockInsertStart => {
                app.buffer.save_undo_snapshot();
                app.visual_block_insert_start();
                app.set_message("-- (block) INSERT --");
            }

            VimAction::VisualBlockInsertEnd => {
                app.buffer.save_undo_snapshot();
                app.visual_block_insert_end();
                app.set_message("-- (block) INSERT --");
            }

            VimAction::VisualLineInsertStart => {
                app.buffer.save_undo_snapshot();
                app.visual_line_insert_start();
                app.set_message("-- INSERT --");
            }

            VimAction::VisualLineInsertEnd => {
                app.buffer.save_undo_snapshot();
                app.visual_line_insert_end();
                app.set_message("-- INSERT --");
            }

            VimAction::ExitToNormal => {
                app.visual_target_display_col = None;
                app.exit_visual_mode();
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
                } else if app.view == View::Editor && !app.search_query.is_empty() {
                    // Enter pressed — jump to first match
                    let (row, col) = app.buffer.cursor_position();
                    if let Some((r, c)) = app.buffer.find_next(&app.search_query, row, col) {
                        app.buffer.set_cursor(r, c);
                        app.set_message(&format!("/{}", app.search_query));
                    } else {
                        app.set_message(&format!("Pattern not found: {}", app.search_query));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

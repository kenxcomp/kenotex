use std::path::PathBuf;

use anyhow::Result;
use uuid::Uuid;

use crate::atoms::storage::file_watcher::FileEvent;
use crate::atoms::storage::{
    delete_draft, ensure_config_dir, ensure_data_dirs, load_all_drafts, load_config, load_draft,
    resolve_data_dir, save_draft,
};
use crate::molecules::config::ThemeManager;
use crate::molecules::distribution::{dispatch_block, parse_smart_blocks, DispatchResult};
use crate::molecules::editor::{TextBuffer, VimMode};
use crate::molecules::list::{
    classify_event, ArchiveList, DraftList, FileChangeAction, FileChangeTracker,
};
use crate::types::{AppMode, Config, Note, ProcessingStatus, SmartBlock, Theme, View};

pub struct App {
    pub mode: AppMode,
    pub view: View,
    pub config: Config,
    pub theme_manager: ThemeManager,
    pub vim_mode: VimMode,

    pub buffer: TextBuffer,
    pub current_note: Option<Note>,
    pub draft_list: DraftList,
    pub archive_list: ArchiveList,

    pub command_message: String,
    pub search_query: String,

    pub processing_blocks: Vec<SmartBlock>,
    pub processing_index: usize,

    pub show_hints: bool,

    pub should_quit: bool,
    pub dirty: bool,
    pub external_editor_requested: bool,
    pub last_save: std::time::Instant,

    pub visual_anchor: Option<(usize, usize)>,
    pub last_yank_linewise: bool,

    pub data_dir: PathBuf,
    pub file_change_tracker: FileChangeTracker,
    pub pending_external_reload: Option<String>,
    pub pending_delete_title: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        ensure_config_dir()?;

        let config = load_config()?;
        let data_dir = resolve_data_dir(config.general.data_dir.as_deref());
        ensure_data_dirs(&data_dir)?;

        let theme_manager = ThemeManager::with_theme(&config.general.theme);

        let vim_mode = VimMode::with_config(config.keyboard.clone());

        let drafts = load_all_drafts(&data_dir, false)?;
        let archives = load_all_drafts(&data_dir, true)?;

        let draft_list = DraftList::new(drafts);
        let archive_list = ArchiveList::new(archives);

        let (buffer, current_note) = if let Some(note) = draft_list.selected_note() {
            (
                TextBuffer::from_string(&note.content),
                Some(note.clone()),
            )
        } else {
            (TextBuffer::new(), None)
        };

        let show_hints = config.general.show_hints;

        Ok(Self {
            mode: AppMode::Normal,
            view: View::Editor,
            config,
            theme_manager,
            vim_mode,
            buffer,
            current_note,
            draft_list,
            archive_list,
            command_message: String::new(),
            search_query: String::new(),
            processing_blocks: Vec::new(),
            processing_index: 0,
            show_hints,
            should_quit: false,
            dirty: false,
            external_editor_requested: false,
            last_save: std::time::Instant::now(),
            visual_anchor: None,
            last_yank_linewise: false,
            data_dir,
            file_change_tracker: FileChangeTracker::new(),
            pending_external_reload: None,
            pending_delete_title: None,
        })
    }

    pub fn theme(&self) -> &Theme {
        self.theme_manager.current()
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn set_view(&mut self, view: View) {
        self.view = view;
    }

    pub fn set_message(&mut self, msg: &str) {
        self.command_message = msg.to_string();
    }

    pub fn clear_message(&mut self) {
        self.command_message.clear();
    }

    pub fn cycle_theme(&mut self) {
        self.theme_manager.cycle_next();
        self.set_message(&format!("Theme: {}", self.theme().name));
    }

    pub fn toggle_hints(&mut self) {
        self.show_hints = !self.show_hints;
        let msg = if self.show_hints {
            "Hints shown"
        } else {
            "Hints hidden"
        };
        self.set_message(msg);
    }

    pub fn new_note(&mut self) {
        let id = Uuid::new_v4().to_string();
        let note = Note::new(id, "Untitled".to_string(), String::new());

        self.buffer = TextBuffer::new();
        self.current_note = Some(note.clone());
        self.draft_list.add_note(note);
        self.dirty = true;

        self.set_view(View::Editor);
        self.set_mode(AppMode::Insert);
        self.set_message("New note created");
    }

    pub fn save_current_note(&mut self) -> Result<()> {
        if let Some(ref mut note) = self.current_note {
            note.update_content(self.buffer.to_string());
            self.file_change_tracker.record_save(&note.id);
            save_draft(&self.data_dir, note)?;
            self.draft_list.update_note(note);
            self.dirty = false;
            self.last_save = std::time::Instant::now();
            self.set_message("Saved");
        }
        Ok(())
    }

    pub fn auto_save_if_needed(&mut self) -> Result<()> {
        if self.dirty && self.last_save.elapsed().as_millis() >= self.config.general.auto_save_interval_ms as u128 {
            self.save_current_note()?;
        }
        Ok(())
    }

    pub fn open_selected_note(&mut self) {
        match self.view {
            View::DraftList => {
                if let Some(note) = self.draft_list.selected_note() {
                    self.buffer = TextBuffer::from_string(&note.content);
                    self.current_note = Some(note.clone());
                    self.set_view(View::Editor);
                    self.set_mode(AppMode::Normal);
                }
            }
            View::ArchiveList => {
                if let Some(note) = self.archive_list.selected_note() {
                    self.buffer = TextBuffer::from_string(&note.content);
                    self.current_note = Some(note.clone());
                    self.set_view(View::Editor);
                    self.set_mode(AppMode::Normal);
                }
            }
            _ => {}
        }
    }

    pub fn archive_selected_note(&mut self) -> Result<()> {
        if self.view == View::DraftList {
            if let Some(mut note) = self.draft_list.remove_selected() {
                note.is_archived = true;
                let old_id = note.id.clone();
                delete_draft(&self.data_dir, &old_id, false)?;
                save_draft(&self.data_dir, &note)?;

                let archives = load_all_drafts(&self.data_dir, true)?;
                self.archive_list.update_notes(archives);

                self.set_message("Note archived");
            }
        }
        Ok(())
    }

    pub fn restore_selected_note(&mut self) -> Result<()> {
        if self.view == View::ArchiveList {
            if let Some(mut note) = self.archive_list.remove_selected() {
                note.is_archived = false;
                let old_id = note.id.clone();
                delete_draft(&self.data_dir, &old_id, true)?;
                save_draft(&self.data_dir, &note)?;

                let drafts = load_all_drafts(&self.data_dir, false)?;
                self.draft_list.update_notes(drafts);

                self.set_message("Note restored");
            }
        }
        Ok(())
    }

    pub fn delete_selected_note(&mut self) -> Result<()> {
        match self.view {
            View::DraftList => {
                if let Some(note) = self.draft_list.remove_selected() {
                    delete_draft(&self.data_dir, &note.id, false)?;
                    self.set_message("Note deleted");
                }
            }
            View::ArchiveList => {
                if let Some(note) = self.archive_list.remove_selected() {
                    delete_draft(&self.data_dir, &note.id, true)?;
                    self.set_message("Note deleted");
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn request_delete(&mut self) {
        let title = match self.view {
            View::DraftList => self.draft_list.selected_note().map(|n| n.title.clone()),
            View::ArchiveList => self.archive_list.selected_note().map(|n| n.title.clone()),
            _ => None,
        };
        if let Some(title) = title {
            self.pending_delete_title = Some(title);
            self.set_mode(AppMode::ConfirmDelete);
        }
    }

    pub fn confirm_delete(&mut self) -> Result<()> {
        self.delete_selected_note()?;
        self.pending_delete_title = None;
        self.set_mode(AppMode::Normal);
        Ok(())
    }

    pub fn cancel_delete(&mut self) {
        self.pending_delete_title = None;
        self.set_mode(AppMode::Normal);
        self.clear_message();
    }

    pub fn start_processing(&mut self) {
        if self.current_note.is_none() {
            return;
        }

        // Parse from current buffer so byte offsets match the live content
        let buffer_content = self.buffer.to_string();
        let blocks = parse_smart_blocks(&buffer_content);
        if blocks.is_empty() {
            self.set_message("No blocks to process");
            return;
        }

        self.processing_blocks = blocks;
        self.processing_index = 0;
        self.set_mode(AppMode::Processing);
    }

    pub fn process_next_block(&mut self) -> bool {
        if self.processing_index < self.processing_blocks.len() {
            let result = dispatch_block(
                &self.processing_blocks[self.processing_index],
                &self.config.destinations,
            );
            self.processing_blocks[self.processing_index].status = match result {
                DispatchResult::Sent => ProcessingStatus::Sent,
                DispatchResult::Skipped => ProcessingStatus::Skipped,
                DispatchResult::Failed(ref msg) => {
                    self.set_message(&format!("Block failed: {}", msg));
                    ProcessingStatus::Failed
                }
            };
            self.processing_index += 1;
            true
        } else {
            false
        }
    }

    pub fn finish_processing(&mut self) {
        // Collect ranges of Sent blocks for comment wrapping
        let mut sent_ranges: Vec<(usize, usize)> = self
            .processing_blocks
            .iter()
            .filter(|b| b.status == ProcessingStatus::Sent)
            .filter_map(|b| b.original_range)
            .collect();

        // Sort by start offset descending so we can replace from end to start
        // without invalidating earlier offsets
        sent_ranges.sort_by(|a, b| b.0.cmp(&a.0));

        if !sent_ranges.is_empty() {
            let mut content = self.buffer.to_string();
            for (start, end) in &sent_ranges {
                let block_text = &content[*start..*end];
                let commented = format!("<!-- {} -->", block_text);
                content.replace_range(*start..*end, &commented);
            }
            self.buffer = TextBuffer::from_string(&content);
            self.dirty = true;
        }

        // Build summary message
        let sent_count = self
            .processing_blocks
            .iter()
            .filter(|b| b.status == ProcessingStatus::Sent)
            .count();
        let skipped_count = self
            .processing_blocks
            .iter()
            .filter(|b| b.status == ProcessingStatus::Skipped)
            .count();
        let failed_count = self
            .processing_blocks
            .iter()
            .filter(|b| b.status == ProcessingStatus::Failed)
            .count();

        let summary = format!(
            "Processing complete: {} sent, {} skipped, {} failed",
            sent_count, skipped_count, failed_count
        );

        self.processing_blocks.clear();
        self.processing_index = 0;
        self.set_mode(AppMode::Normal);
        self.set_message(&summary);
    }

    pub fn refresh_lists(&mut self) -> Result<()> {
        let drafts = load_all_drafts(&self.data_dir, false)?;
        let archives = load_all_drafts(&self.data_dir, true)?;
        self.draft_list.update_notes(drafts);
        self.archive_list.update_notes(archives);
        Ok(())
    }

    pub fn handle_file_event(&mut self, event: FileEvent) -> Result<()> {
        let known_ids: Vec<String> = self
            .draft_list
            .all_note_ids()
            .into_iter()
            .chain(self.archive_list.all_note_ids())
            .collect();

        let action = classify_event(&event, &self.file_change_tracker, &known_ids);

        match action {
            FileChangeAction::Suppressed => {}
            FileChangeAction::ReloadNote { id, is_archived } => {
                let is_current = self
                    .current_note
                    .as_ref()
                    .is_some_and(|n| n.id == id);

                if is_current {
                    if self.dirty {
                        self.pending_external_reload = Some(id);
                        self.set_message(
                            "File changed externally. Ctrl+L to reload, or save to keep yours.",
                        );
                    } else {
                        self.reload_current_note_from_disk()?;
                        self.set_message("File reloaded");
                    }
                } else if let Ok(updated_note) = load_draft(&self.data_dir, &id, is_archived) {
                    if is_archived {
                        self.archive_list.update_single_note(updated_note);
                    } else {
                        self.draft_list.update_note(&updated_note);
                    }
                }
            }
            FileChangeAction::NewNote { .. } | FileChangeAction::DeletedNote { .. } => {
                self.refresh_lists()?;

                if let FileChangeAction::DeletedNote { ref id, .. } = action {
                    let is_current = self
                        .current_note
                        .as_ref()
                        .is_some_and(|n| n.id == *id);
                    if is_current {
                        self.buffer = TextBuffer::new();
                        self.current_note = None;
                        self.dirty = false;
                        self.set_view(View::DraftList);
                        self.set_message("Current note deleted externally");
                    }
                }
            }
        }

        self.file_change_tracker.cleanup();
        Ok(())
    }

    pub fn reload_current_note_from_disk(&mut self) -> Result<()> {
        if let Some(ref note) = self.current_note {
            let id = note.id.clone();
            let is_archived = note.is_archived;
            match load_draft(&self.data_dir, &id, is_archived) {
                Ok(reloaded) => {
                    let current_content = self.buffer.to_string();
                    if reloaded.content == current_content {
                        // Content unchanged (e.g. self-save detected by file watcher).
                        // Update metadata only; keep buffer and cursor intact.
                        self.current_note = Some(reloaded.clone());
                        self.pending_external_reload = None;
                    } else {
                        // Genuine external edit: replace buffer but preserve cursor
                        let (old_row, old_col) = self.buffer.cursor_position();
                        self.buffer = TextBuffer::from_string(&reloaded.content);
                        self.buffer.set_cursor(old_row, old_col);
                        self.current_note = Some(reloaded.clone());
                        self.dirty = false;
                        self.pending_external_reload = None;
                    }
                    if is_archived {
                        self.archive_list.update_single_note(reloaded);
                    } else {
                        self.draft_list.update_note(&reloaded);
                    }
                }
                Err(_) => {
                    self.set_message("Failed to reload note from disk");
                }
            }
        }
        Ok(())
    }

    pub fn scroll_offset(&self, area_width: u16, area_height: u16) -> u16 {
        use crate::atoms::widgets::wrap_calc;

        let (cursor_row, cursor_col) = self.buffer.cursor_position();
        let inner_width = area_width.saturating_sub(2);
        let inner_height = area_height.saturating_sub(2);

        let content = self.buffer.to_string();
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let vpos =
            wrap_calc::visual_cursor_position(&lines, cursor_row, cursor_col, inner_width);
        let cursor_display_row = vpos.rows_before + vpos.wrap_row;

        if inner_height == 0 {
            return 0;
        }

        if cursor_display_row >= inner_height {
            cursor_display_row - inner_height + 5
        } else {
            0
        }
    }

    /// Returns the visual selection as (start, end) sorted, or None if not in visual mode.
    pub fn visual_selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.visual_anchor?;
        let cursor = self.buffer.cursor_position();
        if anchor <= cursor {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    pub fn request_external_editor(&mut self) {
        self.external_editor_requested = true;
    }

    pub fn apply_external_editor_result(&mut self, new_content: String) {
        self.buffer = TextBuffer::from_string(&new_content);
        self.buffer.set_cursor(0, 0);
        self.set_mode(AppMode::Normal);
        self.dirty = true;
        self.set_message("Buffer updated from external editor");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to initialize app")
    }
}

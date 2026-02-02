use anyhow::Result;
use uuid::Uuid;

use crate::atoms::storage::{
    delete_draft, ensure_config_dir, ensure_drafts_dir, load_all_drafts, load_config, save_draft,
};
use crate::molecules::config::ThemeManager;
use crate::molecules::distribution::parse_smart_blocks;
use crate::molecules::editor::{TextBuffer, VimMode};
use crate::molecules::list::{ArchiveList, DraftList};
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
}

impl App {
    pub fn new() -> Result<Self> {
        ensure_config_dir()?;
        ensure_drafts_dir()?;

        let config = load_config()?;
        let theme_manager = ThemeManager::with_theme(&config.general.theme);

        let vim_mode = VimMode::with_config(config.keyboard.clone());

        let drafts = load_all_drafts(false)?;
        let archives = load_all_drafts(true)?;

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
        })
    }

    pub fn theme(&self) -> &Theme {
        self.theme_manager.current()
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        if mode != AppMode::Search {
            self.search_query.clear();
        }
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
            save_draft(note)?;
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
                delete_draft(&old_id, false)?;
                save_draft(&note)?;

                let archives = load_all_drafts(true)?;
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
                delete_draft(&old_id, true)?;
                save_draft(&note)?;

                let drafts = load_all_drafts(false)?;
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
                    delete_draft(&note.id, false)?;
                    self.set_message("Note deleted");
                }
            }
            View::ArchiveList => {
                if let Some(note) = self.archive_list.remove_selected() {
                    delete_draft(&note.id, true)?;
                    self.set_message("Note deleted");
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn start_processing(&mut self) {
        if let Some(ref note) = self.current_note {
            let blocks = parse_smart_blocks(&note.content);
            if blocks.is_empty() {
                self.set_message("No blocks to process");
                return;
            }

            self.processing_blocks = blocks;
            self.processing_index = 0;
            self.set_mode(AppMode::Processing);
        }
    }

    pub fn process_next_block(&mut self) -> bool {
        if self.processing_index < self.processing_blocks.len() {
            self.processing_blocks[self.processing_index].status = ProcessingStatus::Sent;
            self.processing_index += 1;
            true
        } else {
            false
        }
    }

    pub fn finish_processing(&mut self) {
        self.processing_blocks.clear();
        self.processing_index = 0;
        self.set_mode(AppMode::Normal);
        self.set_message("Processing complete");
    }

    pub fn refresh_lists(&mut self) -> Result<()> {
        let drafts = load_all_drafts(false)?;
        let archives = load_all_drafts(true)?;
        self.draft_list.update_notes(drafts);
        self.archive_list.update_notes(archives);
        Ok(())
    }

    pub fn scroll_offset(&self) -> u16 {
        let cursor_row = self.buffer.cursor_position().0 as u16;
        let visible_height = 20;

        if cursor_row >= visible_height {
            cursor_row - visible_height + 5
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

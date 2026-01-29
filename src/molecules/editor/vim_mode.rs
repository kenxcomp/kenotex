use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::types::{AppMode, KeyboardConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimAction {
    None,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    InsertMode,
    InsertModeAppend,
    InsertModeLineEnd,
    InsertModeLineStart,
    InsertLineBelow,
    InsertLineAbove,
    DeleteChar,
    DeleteLine,
    Backspace,
    InsertChar(char),
    InsertNewline,
    EnterVisualMode,
    ExitToNormal,
    Undo,
    Redo,
    LeaderKey,
    LeaderSave,
    LeaderList,
    LeaderNew,
    LeaderProcess,
    CycleTheme,
    Search,
    ExternalEditor,
    Quit,
}

#[derive(Debug, Clone)]
pub struct VimMode {
    leader_pending: bool,
    keys: KeyboardConfig,
}

impl Default for VimMode {
    fn default() -> Self {
        Self::new()
    }
}

impl VimMode {
    pub fn new() -> Self {
        Self {
            leader_pending: false,
            keys: KeyboardConfig::default(),
        }
    }

    pub fn with_config(config: KeyboardConfig) -> Self {
        Self {
            leader_pending: false,
            keys: config,
        }
    }

    pub fn is_leader_pending(&self) -> bool {
        self.leader_pending
    }

    pub fn clear_leader(&mut self) {
        self.leader_pending = false;
    }

    fn key_matches(&self, c: char, binding: &str) -> bool {
        c.to_string() == binding
    }

    pub fn handle_key(&mut self, key: KeyEvent, mode: AppMode) -> VimAction {
        match mode {
            AppMode::Normal => self.handle_normal_mode(key),
            AppMode::Insert => self.handle_insert_mode(key),
            AppMode::Visual => self.handle_visual_mode(key),
            AppMode::Search => self.handle_search_mode(key),
            AppMode::Processing => VimAction::None,
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> VimAction {
        if self.leader_pending {
            self.leader_pending = false;
            return match key.code {
                KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_process) => {
                    VimAction::LeaderProcess
                }
                KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_list) => {
                    VimAction::LeaderList
                }
                KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_new) => {
                    VimAction::LeaderNew
                }
                KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_save) => {
                    VimAction::LeaderSave
                }
                _ => VimAction::None,
            };
        }

        match key.code {
            // Leader key
            KeyCode::Char(' ') => {
                self.leader_pending = true;
                VimAction::LeaderKey
            }

            // Navigation - arrow keys always work
            KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Right => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Home => VimAction::MoveLineStart,
            KeyCode::End => VimAction::MoveLineEnd,

            // Navigation - configurable keys
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_left) => VimAction::MoveLeft,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_right) => VimAction::MoveRight,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_up) => VimAction::MoveUp,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_down) => VimAction::MoveDown,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.word_forward) => {
                VimAction::MoveWordForward
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.word_backward) => {
                VimAction::MoveWordBackward
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.line_start) => {
                VimAction::MoveLineStart
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.line_end) => {
                VimAction::MoveLineEnd
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.file_end) => {
                VimAction::MoveFileEnd
            }
            KeyCode::Char(c)
                if self.key_matches(c, &self.keys.file_start)
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                VimAction::MoveFileStart
            }

            // Insert mode entry
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert) => VimAction::InsertMode,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert_append) => {
                VimAction::InsertModeAppend
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert_line_end) => {
                VimAction::InsertModeLineEnd
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert_line_start) => {
                VimAction::InsertModeLineStart
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert_line_below) => {
                VimAction::InsertLineBelow
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.insert_line_above) => {
                VimAction::InsertLineAbove
            }

            // Editing
            KeyCode::Char(c) if self.key_matches(c, &self.keys.delete_char) => {
                VimAction::DeleteChar
            }
            KeyCode::Char(c)
                if self.key_matches(c, &self.keys.delete_line)
                    && key.modifiers.contains(KeyModifiers::NONE) =>
            {
                VimAction::DeleteLine
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.undo) => VimAction::Undo,
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Redo,

            // Modes
            KeyCode::Char(c) if self.key_matches(c, &self.keys.visual_mode) => {
                VimAction::EnterVisualMode
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.search) => VimAction::Search,
            KeyCode::Char('f') => VimAction::Search, // Alternative search key

            // Other
            KeyCode::Char(c) if self.key_matches(c, &self.keys.cycle_theme) => {
                VimAction::CycleTheme
            }

            // External editor (Ctrl+G)
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::ExternalEditor
            }

            // Quit
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Quit,

            KeyCode::Esc => VimAction::ExitToNormal,

            _ => VimAction::None,
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc => VimAction::ExitToNormal,
            KeyCode::Backspace => VimAction::Backspace,
            KeyCode::Delete => VimAction::DeleteChar,
            KeyCode::Enter => VimAction::InsertNewline,
            KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Right => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Home => VimAction::MoveLineStart,
            KeyCode::End => VimAction::MoveLineEnd,
            KeyCode::Tab => VimAction::InsertChar('\t'),
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => VimAction::ExitToNormal,
                        'g' => VimAction::ExternalEditor,
                        _ => VimAction::None,
                    }
                } else {
                    VimAction::InsertChar(c)
                }
            }
            _ => VimAction::None,
        }
    }

    fn handle_visual_mode(&mut self, key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc => VimAction::ExitToNormal,
            KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Right => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_left) => VimAction::MoveLeft,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_right) => VimAction::MoveRight,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_up) => VimAction::MoveUp,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.move_down) => VimAction::MoveDown,
            _ => VimAction::None,
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => VimAction::ExitToNormal,
            KeyCode::Backspace => VimAction::Backspace,
            KeyCode::Char(c) => VimAction::InsertChar(c),
            _ => VimAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_key_sequence() {
        let mut vim = VimMode::new();

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderKey);
        assert!(vim.is_leader_pending());

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderProcess);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_insert_mode() {
        let mut vim = VimMode::new();

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::InsertMode);

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            AppMode::Insert,
        );
        assert_eq!(action, VimAction::InsertChar('a'));
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::types::AppMode;

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

#[derive(Debug, Clone, Default)]
pub struct VimMode {
    leader_pending: bool,
    direction_up: String,
    direction_down: String,
}

impl VimMode {
    pub fn new() -> Self {
        Self {
            leader_pending: false,
            direction_up: "k".to_string(),
            direction_down: "j".to_string(),
        }
    }

    pub fn with_keybindings(direction_up: String, direction_down: String) -> Self {
        Self {
            leader_pending: false,
            direction_up,
            direction_down,
        }
    }

    pub fn is_leader_pending(&self) -> bool {
        self.leader_pending
    }

    pub fn clear_leader(&mut self) {
        self.leader_pending = false;
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
                KeyCode::Char('s') => VimAction::LeaderProcess,
                KeyCode::Char('l') => VimAction::LeaderList,
                KeyCode::Char('n') => VimAction::LeaderNew,
                KeyCode::Char('w') => VimAction::LeaderSave,
                _ => VimAction::None,
            };
        }

        match key.code {
            KeyCode::Char(' ') => {
                self.leader_pending = true;
                VimAction::LeaderKey
            }

            KeyCode::Char('h') | KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Char('l') | KeyCode::Right => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Char(c) if c.to_string() == self.direction_up => VimAction::MoveUp,
            KeyCode::Char(c) if c.to_string() == self.direction_down => VimAction::MoveDown,

            KeyCode::Char('w') => VimAction::MoveWordForward,
            KeyCode::Char('b') => VimAction::MoveWordBackward,
            KeyCode::Char('0') | KeyCode::Home => VimAction::MoveLineStart,
            KeyCode::Char('$') | KeyCode::End => VimAction::MoveLineEnd,
            KeyCode::Char('G') => VimAction::MoveFileEnd,
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::ExternalEditor
            }
            KeyCode::Char('g') => VimAction::MoveFileStart,

            KeyCode::Char('i') => VimAction::InsertMode,
            KeyCode::Char('a') => VimAction::InsertModeAppend,
            KeyCode::Char('A') => VimAction::InsertModeLineEnd,
            KeyCode::Char('I') => VimAction::InsertModeLineStart,
            KeyCode::Char('o') => VimAction::InsertLineBelow,
            KeyCode::Char('O') => VimAction::InsertLineAbove,

            KeyCode::Char('x') => VimAction::DeleteChar,
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::NONE) => {
                VimAction::DeleteLine
            }

            KeyCode::Char('v') => VimAction::EnterVisualMode,

            KeyCode::Char('u') if self.direction_up != "u" => VimAction::Undo,
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Redo,

            KeyCode::Char('T') => VimAction::CycleTheme,
            KeyCode::Char('/') | KeyCode::Char('f') => VimAction::Search,

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
            KeyCode::Char('h') | KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Char('l') | KeyCode::Right => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Char(c) if c.to_string() == self.direction_up => VimAction::MoveUp,
            KeyCode::Char(c) if c.to_string() == self.direction_down => VimAction::MoveDown,
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

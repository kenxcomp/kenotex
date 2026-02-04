use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::types::{AppMode, KeyboardConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Line,
    WordForward,
    WordBackward,
    LineEnd,
    LineStart,
    FileEnd,
    FileStart,
}

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
    InsertTab,
    Indent,
    Dedent,
    InsertNewline,
    EnterVisualMode,
    ExitToNormal,
    Undo,
    Redo,
    LeaderKey,
    LeaderList,
    LeaderNew,
    LeaderProcess,
    ToggleHints,
    InsertCheckbox,
    ToggleCheckbox,
    CycleTheme,
    Search,
    SearchNext,
    SearchPrev,
    ClearSearch,
    ExternalEditor,
    Quit,
    Delete(Motion),
    Yank(Motion),
    VisualDelete,
    VisualYank,
    PasteAfter,
    PasteBefore,
    ReloadBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LeaderState {
    Inactive,
    AwaitingFirst,
    AwaitingSecond(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatorPending {
    None,
    Delete,
    Yank,
}

#[derive(Debug, Clone)]
pub struct VimMode {
    leader_state: LeaderState,
    operator_state: OperatorPending,
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
            leader_state: LeaderState::Inactive,
            operator_state: OperatorPending::None,
            keys: KeyboardConfig::default(),
        }
    }

    pub fn with_config(config: KeyboardConfig) -> Self {
        Self {
            leader_state: LeaderState::Inactive,
            operator_state: OperatorPending::None,
            keys: config,
        }
    }

    pub fn is_leader_pending(&self) -> bool {
        self.leader_state != LeaderState::Inactive
    }

    pub fn clear_leader(&mut self) {
        self.leader_state = LeaderState::Inactive;
    }

    pub fn is_operator_pending(&self) -> bool {
        self.operator_state != OperatorPending::None
    }

    pub fn clear_operator(&mut self) {
        self.operator_state = OperatorPending::None;
    }

    fn key_matches(&self, c: char, binding: &str) -> bool {
        c.to_string() == binding
    }

    fn key_event_matches(&self, key: &KeyEvent, binding: &str) -> bool {
        if let Some(ch_str) = binding.strip_prefix("ctrl+") {
            if let Some(c) = ch_str.chars().next() {
                if ch_str.len() == 1 {
                    return key.code == KeyCode::Char(c)
                        && key.modifiers.contains(KeyModifiers::CONTROL);
                }
            }
            return false;
        }
        if let KeyCode::Char(c) = key.code {
            c.to_string() == binding
        } else {
            false
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, mode: AppMode) -> VimAction {
        match mode {
            AppMode::Normal => self.handle_normal_mode(key),
            AppMode::Insert => self.handle_insert_mode(key),
            AppMode::Visual => self.handle_visual_mode(key),
            AppMode::Search => self.handle_search_mode(key),
            AppMode::Processing | AppMode::ConfirmDelete => VimAction::None,
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> VimAction {
        match &self.leader_state {
            LeaderState::AwaitingSecond(first) => {
                let first = *first;
                self.leader_state = LeaderState::Inactive;
                return match (first, key.code) {
                    ('n', KeyCode::Char('n')) => VimAction::LeaderNew,
                    ('m', KeyCode::Char('c')) => VimAction::InsertCheckbox,
                    _ => VimAction::None,
                };
            }
            LeaderState::AwaitingFirst => {
                return match key.code {
                    KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_process) => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::LeaderProcess
                    }
                    KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_list) => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::LeaderList
                    }
                    KeyCode::Char(c) if self.key_matches(c, &self.keys.leader_quit) => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::Quit
                    }
                    KeyCode::Char('d') => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::ToggleCheckbox
                    }
                    KeyCode::Char('h') => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::ToggleHints
                    }
                    KeyCode::Char('n') => {
                        self.leader_state = LeaderState::AwaitingSecond('n');
                        VimAction::None
                    }
                    KeyCode::Char('m') => {
                        self.leader_state = LeaderState::AwaitingSecond('m');
                        VimAction::None
                    }
                    _ => {
                        self.leader_state = LeaderState::Inactive;
                        VimAction::None
                    }
                };
            }
            LeaderState::Inactive => {}
        }

        // Operator-pending: resolve motion
        if self.operator_state != OperatorPending::None {
            let op = self.operator_state;
            self.operator_state = OperatorPending::None;
            if let Some(motion) = self.resolve_motion(key) {
                return match op {
                    OperatorPending::Delete => VimAction::Delete(motion),
                    OperatorPending::Yank => VimAction::Yank(motion),
                    OperatorPending::None => unreachable!(),
                };
            }
            return VimAction::None;
        }

        match key.code {
            // Leader key
            KeyCode::Char(' ') => {
                self.leader_state = LeaderState::AwaitingFirst;
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
                self.operator_state = OperatorPending::Delete;
                VimAction::None
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.yank) => {
                self.operator_state = OperatorPending::Yank;
                VimAction::None
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.paste_after) => {
                VimAction::PasteAfter
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.paste_before) => {
                VimAction::PasteBefore
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.undo) => VimAction::Undo,
            KeyCode::Char(_) if self.key_event_matches(&key, &self.keys.redo) => VimAction::Redo,

            // Modes
            KeyCode::Char(c) if self.key_matches(c, &self.keys.visual_mode) => {
                VimAction::EnterVisualMode
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.search) => VimAction::Search,
            KeyCode::Char('f') => VimAction::Search, // Alternative search key
            KeyCode::Char(c) if self.key_matches(c, &self.keys.search_next) => VimAction::SearchNext,
            KeyCode::Char(c) if self.key_matches(c, &self.keys.search_prev) => VimAction::SearchPrev,

            // Other
            KeyCode::Char(c) if self.key_matches(c, &self.keys.cycle_theme) => {
                VimAction::CycleTheme
            }

            // Reload buffer from disk (Ctrl+L)
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::ReloadBuffer
            }

            // External editor (Ctrl+G)
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::ExternalEditor
            }

            // Quit
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Quit,

            KeyCode::Esc => VimAction::ExitToNormal,

            KeyCode::Char('>') => VimAction::Indent,
            KeyCode::Char('<') => VimAction::Dedent,

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
            KeyCode::Tab => VimAction::InsertTab,
            KeyCode::BackTab => VimAction::Dedent,
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
            KeyCode::Char(c) if self.key_matches(c, &self.keys.file_start) => {
                VimAction::MoveFileStart
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.file_end) => {
                VimAction::MoveFileEnd
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.delete_line) => {
                VimAction::VisualDelete
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.yank) => VimAction::VisualYank,
            KeyCode::Char('>') => VimAction::Indent,
            KeyCode::Char('<') => VimAction::Dedent,
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

    fn resolve_motion(&self, key: KeyEvent) -> Option<Motion> {
        match key.code {
            KeyCode::Char(c) if self.key_matches(c, &self.keys.delete_line) => {
                Some(Motion::Line)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.yank) => Some(Motion::Line),
            KeyCode::Char(c) if self.key_matches(c, &self.keys.word_forward) => {
                Some(Motion::WordForward)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.word_backward) => {
                Some(Motion::WordBackward)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.line_end) => {
                Some(Motion::LineEnd)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.line_start) => {
                Some(Motion::LineStart)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.file_end) => {
                Some(Motion::FileEnd)
            }
            KeyCode::Char(c) if self.key_matches(c, &self.keys.file_start) => {
                Some(Motion::FileStart)
            }
            _ => None,
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
    fn test_leader_toggle_hints() {
        let mut vim = VimMode::new();

        // Space -> AwaitingFirst
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderKey);
        assert!(vim.is_leader_pending());

        // 'h' -> ToggleHints
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::ToggleHints);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_leader_multi_char_cancel() {
        let mut vim = VimMode::new();

        // Space -> AwaitingFirst
        vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );

        // 'n' -> AwaitingSecond('n')
        vim.handle_key(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert!(vim.is_leader_pending());

        // 'x' (invalid second char) -> cancel
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_leader_insert_checkbox() {
        let mut vim = VimMode::new();

        // Space -> AwaitingFirst
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderKey);
        assert!(vim.is_leader_pending());

        // 'm' -> AwaitingSecond('m')
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(vim.is_leader_pending());

        // 'c' -> InsertCheckbox
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::InsertCheckbox);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_leader_toggle_checkbox() {
        let mut vim = VimMode::new();

        // Space -> AwaitingFirst
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderKey);

        // 'd' -> ToggleCheckbox
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::ToggleCheckbox);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_leader_m_invalid_second() {
        let mut vim = VimMode::new();

        vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        vim.handle_key(
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            AppMode::Normal,
        );

        // 'z' (invalid) -> None, leader cancelled
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(!vim.is_leader_pending());
    }

    #[test]
    fn test_leader_new_note() {
        let mut vim = VimMode::new();

        // Space -> AwaitingFirst
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderKey);
        assert!(vim.is_leader_pending());

        // 'n' -> AwaitingSecond('n')
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(vim.is_leader_pending());

        // 'n' -> LeaderNew
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::LeaderNew);
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

    #[test]
    fn test_dd_deletes_line() {
        let mut vim = VimMode::new();
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(vim.is_operator_pending());

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::Delete(Motion::Line));
    }

    #[test]
    fn test_dw_delete_word() {
        let mut vim = VimMode::new();
        vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::Delete(Motion::WordForward));
    }

    #[test]
    fn test_yy_yank_line() {
        let mut vim = VimMode::new();
        vim.handle_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::Yank(Motion::Line));
    }

    #[test]
    fn test_operator_cancel_on_invalid() {
        let mut vim = VimMode::new();
        vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::None);
        assert!(!vim.is_operator_pending());
    }

    #[test]
    fn test_visual_d() {
        let mut vim = VimMode::new();
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            AppMode::Visual,
        );
        assert_eq!(action, VimAction::VisualDelete);
    }

    #[test]
    fn test_visual_y() {
        let mut vim = VimMode::new();
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            AppMode::Visual,
        );
        assert_eq!(action, VimAction::VisualYank);
    }

    #[test]
    fn test_paste_keys() {
        let mut vim = VimMode::new();
        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::PasteAfter);

        let action = vim.handle_key(
            KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE),
            AppMode::Normal,
        );
        assert_eq!(action, VimAction::PasteBefore);
    }
}

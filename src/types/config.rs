use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub keyboard: KeyboardConfig,
    #[serde(default)]
    pub destinations: Destinations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_leader_key")]
    pub leader_key: String,
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_ms: u64,
    #[serde(default = "default_show_hints")]
    pub show_hints: bool,
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default = "default_file_watch")]
    pub file_watch: bool,
    #[serde(default = "default_file_watch_debounce_ms")]
    pub file_watch_debounce_ms: u64,
    #[serde(default = "default_tab_width")]
    pub tab_width: u8,
}

fn default_theme() -> String {
    "tokyo_night".to_string()
}

fn default_leader_key() -> String {
    " ".to_string()
}

fn default_auto_save_interval() -> u64 {
    5000
}

fn default_show_hints() -> bool {
    true
}

fn default_file_watch() -> bool {
    true
}

fn default_file_watch_debounce_ms() -> u64 {
    300
}

fn default_tab_width() -> u8 {
    4
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            leader_key: default_leader_key(),
            auto_save_interval_ms: default_auto_save_interval(),
            show_hints: default_show_hints(),
            data_dir: None,
            file_watch: default_file_watch(),
            file_watch_debounce_ms: default_file_watch_debounce_ms(),
            tab_width: default_tab_width(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default = "default_layout")]
    pub layout: String,

    // Navigation
    #[serde(default = "default_move_left")]
    pub move_left: String,
    #[serde(default = "default_move_down")]
    pub move_down: String,
    #[serde(default = "default_move_up")]
    pub move_up: String,
    #[serde(default = "default_move_right")]
    pub move_right: String,
    #[serde(default = "default_word_forward")]
    pub word_forward: String,
    #[serde(default = "default_word_backward")]
    pub word_backward: String,
    #[serde(default = "default_line_start")]
    pub line_start: String,
    #[serde(default = "default_line_end")]
    pub line_end: String,
    #[serde(default = "default_first_non_blank")]
    pub first_non_blank: String,
    #[serde(default = "default_word_end")]
    pub word_end: String,
    #[serde(default = "default_file_start")]
    pub file_start: String,
    #[serde(default = "default_file_end")]
    pub file_end: String,
    #[serde(default = "default_scroll_up")]
    pub scroll_up: String,
    #[serde(default = "default_scroll_down")]
    pub scroll_down: String,

    // Insert mode entry
    #[serde(default = "default_insert")]
    pub insert: String,
    #[serde(default = "default_insert_append")]
    pub insert_append: String,
    #[serde(default = "default_insert_line_start")]
    pub insert_line_start: String,
    #[serde(default = "default_insert_line_end")]
    pub insert_line_end: String,
    #[serde(default = "default_insert_line_below")]
    pub insert_line_below: String,
    #[serde(default = "default_insert_line_above")]
    pub insert_line_above: String,

    // Editing
    #[serde(default = "default_delete_char")]
    pub delete_char: String,
    #[serde(default = "default_delete_line")]
    pub delete_line: String,
    #[serde(default = "default_undo")]
    pub undo: String,
    #[serde(default = "default_redo")]
    pub redo: String,
    #[serde(default = "default_yank")]
    pub yank: String,
    #[serde(default = "default_paste_after")]
    pub paste_after: String,
    #[serde(default = "default_paste_before")]
    pub paste_before: String,

    // Modes
    #[serde(default = "default_visual_mode")]
    pub visual_mode: String,
    #[serde(default = "default_visual_line_mode")]
    pub visual_line_mode: String,
    #[serde(default = "default_visual_block_mode")]
    pub visual_block_mode: String,
    #[serde(default = "default_search")]
    pub search: String,
    #[serde(default = "default_search_next")]
    pub search_next: String,
    #[serde(default = "default_search_prev")]
    pub search_prev: String,

    // Other
    #[serde(default = "default_cycle_theme")]
    pub cycle_theme: String,

    // Leader commands
    #[serde(default = "default_leader_process")]
    pub leader_process: String,
    #[serde(default = "default_leader_list")]
    pub leader_list: String,
    #[serde(default = "default_leader_new")]
    pub leader_new: String,
    #[serde(default = "default_leader_quit")]
    pub leader_quit: String,
    #[serde(default = "default_leader_comment")]
    pub leader_comment: String,
    #[serde(default = "default_visual_comment")]
    pub visual_comment: String,

    // Formatting leader keys
    #[serde(default = "default_leader_bold")]
    pub leader_bold: String,
    #[serde(default = "default_leader_italic")]
    pub leader_italic: String,
    #[serde(default = "default_leader_strikethrough")]
    pub leader_strikethrough: String,
    #[serde(default = "default_leader_code")]
    pub leader_code: String,
    #[serde(default = "default_leader_code_block")]
    pub leader_code_block: String,
}

fn default_layout() -> String {
    "qwerty".to_string()
}

// Navigation defaults
fn default_move_left() -> String {
    "h".to_string()
}
fn default_move_down() -> String {
    "j".to_string()
}
fn default_move_up() -> String {
    "k".to_string()
}
fn default_move_right() -> String {
    "l".to_string()
}
fn default_word_forward() -> String {
    "w".to_string()
}
fn default_word_backward() -> String {
    "b".to_string()
}
fn default_line_start() -> String {
    "0".to_string()
}
fn default_line_end() -> String {
    "$".to_string()
}
fn default_first_non_blank() -> String {
    "^".to_string()
}
fn default_word_end() -> String {
    "e".to_string()
}
fn default_file_start() -> String {
    "g".to_string()
}
fn default_file_end() -> String {
    "G".to_string()
}
fn default_scroll_up() -> String {
    "ctrl+u".to_string()
}
fn default_scroll_down() -> String {
    "ctrl+d".to_string()
}

// Insert mode defaults
fn default_insert() -> String {
    "i".to_string()
}
fn default_insert_append() -> String {
    "a".to_string()
}
fn default_insert_line_start() -> String {
    "I".to_string()
}
fn default_insert_line_end() -> String {
    "A".to_string()
}
fn default_insert_line_below() -> String {
    "o".to_string()
}
fn default_insert_line_above() -> String {
    "O".to_string()
}

// Editing defaults
fn default_delete_char() -> String {
    "x".to_string()
}
fn default_delete_line() -> String {
    "d".to_string()
}
fn default_undo() -> String {
    "u".to_string()
}
fn default_redo() -> String {
    "ctrl+r".to_string()
}
fn default_yank() -> String {
    "y".to_string()
}
fn default_paste_after() -> String {
    "p".to_string()
}
fn default_paste_before() -> String {
    "P".to_string()
}

// Mode defaults
fn default_visual_mode() -> String {
    "v".to_string()
}
fn default_visual_line_mode() -> String {
    "V".to_string()
}
fn default_visual_block_mode() -> String {
    "ctrl+v".to_string()
}
fn default_search() -> String {
    "/".to_string()
}
fn default_search_next() -> String {
    "n".to_string()
}
fn default_search_prev() -> String {
    "N".to_string()
}

// Other defaults
fn default_cycle_theme() -> String {
    "T".to_string()
}

// Leader command defaults
fn default_leader_process() -> String {
    "s".to_string()
}
fn default_leader_list() -> String {
    "l".to_string()
}
fn default_leader_new() -> String {
    "nn".to_string()
}
fn default_leader_quit() -> String {
    "q".to_string()
}
fn default_leader_comment() -> String {
    "c".to_string()
}
fn default_visual_comment() -> String {
    "gc".to_string()
}

// Formatting leader key defaults
fn default_leader_bold() -> String {
    "b".to_string()
}
fn default_leader_italic() -> String {
    "i".to_string()
}
fn default_leader_strikethrough() -> String {
    "x".to_string()
}
fn default_leader_code() -> String {
    "c".to_string()
}
fn default_leader_code_block() -> String {
    "C".to_string()
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            layout: default_layout(),
            move_left: default_move_left(),
            move_down: default_move_down(),
            move_up: default_move_up(),
            move_right: default_move_right(),
            word_forward: default_word_forward(),
            word_backward: default_word_backward(),
            line_start: default_line_start(),
            line_end: default_line_end(),
            first_non_blank: default_first_non_blank(),
            word_end: default_word_end(),
            file_start: default_file_start(),
            file_end: default_file_end(),
            scroll_up: default_scroll_up(),
            scroll_down: default_scroll_down(),
            insert: default_insert(),
            insert_append: default_insert_append(),
            insert_line_start: default_insert_line_start(),
            insert_line_end: default_insert_line_end(),
            insert_line_below: default_insert_line_below(),
            insert_line_above: default_insert_line_above(),
            delete_char: default_delete_char(),
            delete_line: default_delete_line(),
            undo: default_undo(),
            redo: default_redo(),
            yank: default_yank(),
            paste_after: default_paste_after(),
            paste_before: default_paste_before(),
            visual_mode: default_visual_mode(),
            visual_line_mode: default_visual_line_mode(),
            visual_block_mode: default_visual_block_mode(),
            search: default_search(),
            search_next: default_search_next(),
            search_prev: default_search_prev(),
            cycle_theme: default_cycle_theme(),
            leader_process: default_leader_process(),
            leader_list: default_leader_list(),
            leader_new: default_leader_new(),
            leader_quit: default_leader_quit(),
            leader_comment: default_leader_comment(),
            visual_comment: default_visual_comment(),
            leader_bold: default_leader_bold(),
            leader_italic: default_leader_italic(),
            leader_strikethrough: default_leader_strikethrough(),
            leader_code: default_leader_code(),
            leader_code_block: default_leader_code_block(),
        }
    }
}

impl KeyboardConfig {
    pub fn colemak() -> Self {
        Self {
            layout: "colemak".to_string(),
            move_up: "u".to_string(),
            move_down: "e".to_string(),
            undo: "z".to_string(), // 'u' is used for move_up
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Destinations {
    #[serde(default)]
    pub reminders: DestinationApp,
    #[serde(default)]
    pub calendar: DestinationApp,
    #[serde(default)]
    pub notes: NotesDestination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationApp {
    #[serde(default = "default_app")]
    pub app: String,
    pub list: Option<String>,
    pub calendar_name: Option<String>,
}

fn default_app() -> String {
    "apple".to_string()
}

impl Default for DestinationApp {
    fn default() -> Self {
        Self {
            app: default_app(),
            list: None,
            calendar_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesDestination {
    #[serde(
        default = "default_notes_app",
        deserialize_with = "deserialize_optional_notes_app",
        serialize_with = "serialize_optional_notes_app"
    )]
    pub app: Option<NotesApp>,
    pub folder: Option<String>,
    pub vault: Option<String>,
}

fn default_notes_app() -> Option<NotesApp> {
    Some(NotesApp::AppleNotes)
}

fn deserialize_optional_notes_app<'de, D>(deserializer: D) -> Result<Option<NotesApp>, D::Error>
where
    D: Deserializer<'de>,
{
    // Accept either a string value or a proper enum variant
    let value = toml::Value::deserialize(deserializer)?;
    match value {
        toml::Value::String(s) if s.is_empty() => Ok(None),
        other => {
            // Try to deserialize as NotesApp
            NotesApp::deserialize(other)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}

fn serialize_optional_notes_app<S>(
    value: &Option<NotesApp>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(app) => app.serialize(serializer),
        None => serializer.serialize_str(""),
    }
}

impl Default for NotesDestination {
    fn default() -> Self {
        Self {
            app: default_notes_app(),
            folder: None,
            vault: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotesApp {
    AppleNotes,
    Bear,
    Obsidian,
}

impl NotesApp {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotesApp::AppleNotes => "Apple Notes",
            NotesApp::Bear => "Bear",
            NotesApp::Obsidian => "Obsidian",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.theme, "tokyo_night");
        assert_eq!(config.general.leader_key, " ");
        assert_eq!(config.general.auto_save_interval_ms, 5000);
        assert!(config.general.show_hints);
        assert!(config.general.data_dir.is_none());
        assert!(config.general.file_watch);
        assert_eq!(config.general.file_watch_debounce_ms, 300);
        assert_eq!(config.general.tab_width, 4);
    }

    #[test]
    fn test_default_keyboard_config() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.layout, "qwerty");
        assert_eq!(kb.move_left, "h");
        assert_eq!(kb.move_down, "j");
        assert_eq!(kb.move_up, "k");
        assert_eq!(kb.move_right, "l");
        assert_eq!(kb.word_forward, "w");
        assert_eq!(kb.word_backward, "b");
        assert_eq!(kb.line_start, "0");
        assert_eq!(kb.line_end, "$");
        assert_eq!(kb.first_non_blank, "^");
        assert_eq!(kb.word_end, "e");
        assert_eq!(kb.scroll_up, "ctrl+u");
        assert_eq!(kb.scroll_down, "ctrl+d");
    }

    #[test]
    fn test_keyboard_config_insert_defaults() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.insert, "i");
        assert_eq!(kb.insert_append, "a");
        assert_eq!(kb.insert_line_start, "I");
        assert_eq!(kb.insert_line_end, "A");
        assert_eq!(kb.insert_line_below, "o");
        assert_eq!(kb.insert_line_above, "O");
    }

    #[test]
    fn test_keyboard_config_editing_defaults() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.delete_char, "x");
        assert_eq!(kb.delete_line, "d");
        assert_eq!(kb.undo, "u");
        assert_eq!(kb.redo, "ctrl+r");
        assert_eq!(kb.yank, "y");
        assert_eq!(kb.paste_after, "p");
        assert_eq!(kb.paste_before, "P");
    }

    #[test]
    fn test_keyboard_config_mode_defaults() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.visual_mode, "v");
        assert_eq!(kb.visual_line_mode, "V");
        assert_eq!(kb.visual_block_mode, "ctrl+v");
        assert_eq!(kb.search, "/");
        assert_eq!(kb.search_next, "n");
        assert_eq!(kb.search_prev, "N");
    }

    #[test]
    fn test_keyboard_config_leader_defaults() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.leader_process, "s");
        assert_eq!(kb.leader_list, "l");
        assert_eq!(kb.leader_new, "nn");
        assert_eq!(kb.leader_quit, "q");
        assert_eq!(kb.leader_comment, "c");
        assert_eq!(kb.visual_comment, "gc");
    }

    #[test]
    fn test_keyboard_config_format_defaults() {
        let kb = KeyboardConfig::default();
        assert_eq!(kb.leader_bold, "b");
        assert_eq!(kb.leader_italic, "i");
        assert_eq!(kb.leader_strikethrough, "x");
        assert_eq!(kb.leader_code, "c");
        assert_eq!(kb.leader_code_block, "C");
    }

    #[test]
    fn test_colemak_keyboard() {
        let kb = KeyboardConfig::colemak();
        assert_eq!(kb.layout, "colemak");
        assert_eq!(kb.move_up, "u");
        assert_eq!(kb.move_down, "e");
        assert_eq!(kb.undo, "z");
        // Other keys should remain default
        assert_eq!(kb.move_left, "h");
        assert_eq!(kb.move_right, "l");
    }

    #[test]
    fn test_default_destinations() {
        let dest = Destinations::default();
        assert_eq!(dest.reminders.app, "apple");
        assert!(dest.reminders.list.is_none());
        assert_eq!(dest.calendar.app, "apple");
        assert!(dest.calendar.calendar_name.is_none());
        assert_eq!(dest.notes.app, Some(NotesApp::AppleNotes));
        assert!(dest.notes.folder.is_none());
        assert!(dest.notes.vault.is_none());
    }

    #[test]
    fn test_notes_app_as_str() {
        assert_eq!(NotesApp::AppleNotes.as_str(), "Apple Notes");
        assert_eq!(NotesApp::Bear.as_str(), "Bear");
        assert_eq!(NotesApp::Obsidian.as_str(), "Obsidian");
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.general.theme, "tokyo_night");
        assert_eq!(parsed.keyboard.layout, "qwerty");
    }

    #[test]
    fn test_config_deserialize_partial_toml() {
        let toml_str = r#"
[general]
theme = "gruvbox"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.theme, "gruvbox");
        // All other fields should be defaults
        assert_eq!(config.general.leader_key, " ");
        assert_eq!(config.keyboard.layout, "qwerty");
    }

    #[test]
    fn test_config_deserialize_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.general.theme, "tokyo_night");
    }

    // =========================================================================
    // Supplementary tests: custom overrides for new navigation fields
    // =========================================================================

    #[test]
    fn test_config_custom_first_non_blank_override() {
        let toml_str = r#"
[keyboard]
first_non_blank = "_"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.first_non_blank, "_");
        // Other navigation fields should remain default
        assert_eq!(config.keyboard.word_end, "e");
        assert_eq!(config.keyboard.scroll_up, "ctrl+u");
        assert_eq!(config.keyboard.scroll_down, "ctrl+d");
    }

    #[test]
    fn test_config_custom_word_end_override() {
        let toml_str = r#"
[keyboard]
word_end = "E"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.word_end, "E");
        assert_eq!(config.keyboard.first_non_blank, "^");
    }

    #[test]
    fn test_config_custom_scroll_up_override() {
        let toml_str = r#"
[keyboard]
scroll_up = "ctrl+b"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.scroll_up, "ctrl+b");
        assert_eq!(config.keyboard.scroll_down, "ctrl+d");
    }

    #[test]
    fn test_config_custom_scroll_down_override() {
        let toml_str = r#"
[keyboard]
scroll_down = "ctrl+f"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.scroll_down, "ctrl+f");
        assert_eq!(config.keyboard.scroll_up, "ctrl+u");
    }

    #[test]
    fn test_config_all_new_nav_fields_override() {
        let toml_str = r#"
[keyboard]
first_non_blank = "_"
word_end = "E"
scroll_up = "ctrl+b"
scroll_down = "ctrl+f"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.first_non_blank, "_");
        assert_eq!(config.keyboard.word_end, "E");
        assert_eq!(config.keyboard.scroll_up, "ctrl+b");
        assert_eq!(config.keyboard.scroll_down, "ctrl+f");
        // Other keys remain default
        assert_eq!(config.keyboard.move_left, "h");
        assert_eq!(config.keyboard.move_right, "l");
    }

    #[test]
    fn test_colemak_new_nav_defaults() {
        let kb = KeyboardConfig::colemak();
        // Colemak overrides: move_up=u, move_down=e, undo=z
        // New nav fields should use their defaults since colemak() uses ..Default::default()
        assert_eq!(kb.first_non_blank, "^");
        assert_eq!(kb.word_end, "e"); // Note: 'e' is also move_down in colemak
        assert_eq!(kb.scroll_up, "ctrl+u");
        assert_eq!(kb.scroll_down, "ctrl+d");
    }

    #[test]
    fn test_config_roundtrip_preserves_new_nav_fields() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.keyboard.first_non_blank, "^");
        assert_eq!(parsed.keyboard.word_end, "e");
        assert_eq!(parsed.keyboard.scroll_up, "ctrl+u");
        assert_eq!(parsed.keyboard.scroll_down, "ctrl+d");
    }

    #[test]
    fn test_config_partial_keyboard_preserves_new_defaults() {
        // If user only sets layout, new fields should still default correctly
        let toml_str = r#"
[keyboard]
layout = "qwerty"
move_left = "h"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard.first_non_blank, "^");
        assert_eq!(config.keyboard.word_end, "e");
        assert_eq!(config.keyboard.scroll_up, "ctrl+u");
        assert_eq!(config.keyboard.scroll_down, "ctrl+d");
    }
}

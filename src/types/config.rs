use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub keyboard: KeyboardConfig,
    #[serde(default)]
    pub destinations: Destinations,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            keyboard: KeyboardConfig::default(),
            destinations: Destinations::default(),
        }
    }
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
    #[serde(default = "default_file_start")]
    pub file_start: String,
    #[serde(default = "default_file_end")]
    pub file_end: String,

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
}

fn default_layout() -> String { "qwerty".to_string() }

// Navigation defaults
fn default_move_left() -> String { "h".to_string() }
fn default_move_down() -> String { "j".to_string() }
fn default_move_up() -> String { "k".to_string() }
fn default_move_right() -> String { "l".to_string() }
fn default_word_forward() -> String { "w".to_string() }
fn default_word_backward() -> String { "b".to_string() }
fn default_line_start() -> String { "0".to_string() }
fn default_line_end() -> String { "$".to_string() }
fn default_file_start() -> String { "g".to_string() }
fn default_file_end() -> String { "G".to_string() }

// Insert mode defaults
fn default_insert() -> String { "i".to_string() }
fn default_insert_append() -> String { "a".to_string() }
fn default_insert_line_start() -> String { "I".to_string() }
fn default_insert_line_end() -> String { "A".to_string() }
fn default_insert_line_below() -> String { "o".to_string() }
fn default_insert_line_above() -> String { "O".to_string() }

// Editing defaults
fn default_delete_char() -> String { "x".to_string() }
fn default_delete_line() -> String { "d".to_string() }
fn default_undo() -> String { "u".to_string() }
fn default_redo() -> String { "ctrl+r".to_string() }
fn default_yank() -> String { "y".to_string() }
fn default_paste_after() -> String { "p".to_string() }
fn default_paste_before() -> String { "P".to_string() }

// Mode defaults
fn default_visual_mode() -> String { "v".to_string() }
fn default_search() -> String { "/".to_string() }
fn default_search_next() -> String { "n".to_string() }
fn default_search_prev() -> String { "N".to_string() }

// Other defaults
fn default_cycle_theme() -> String { "T".to_string() }

// Leader command defaults
fn default_leader_process() -> String { "s".to_string() }
fn default_leader_list() -> String { "l".to_string() }
fn default_leader_new() -> String { "nn".to_string() }
fn default_leader_quit() -> String { "q".to_string() }

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
            file_start: default_file_start(),
            file_end: default_file_end(),
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
            search: default_search(),
            search_next: default_search_next(),
            search_prev: default_search_prev(),
            cycle_theme: default_cycle_theme(),
            leader_process: default_leader_process(),
            leader_list: default_leader_list(),
            leader_new: default_leader_new(),
            leader_quit: default_leader_quit(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destinations {
    #[serde(default)]
    pub reminders: DestinationApp,
    #[serde(default)]
    pub calendar: DestinationApp,
    #[serde(default)]
    pub notes: NotesDestination,
}

impl Default for Destinations {
    fn default() -> Self {
        Self {
            reminders: DestinationApp::default(),
            calendar: DestinationApp::default(),
            notes: NotesDestination::default(),
        }
    }
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

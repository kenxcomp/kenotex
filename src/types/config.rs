use serde::{Deserialize, Serialize};

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

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            leader_key: default_leader_key(),
            auto_save_interval_ms: default_auto_save_interval(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_direction_up")]
    pub direction_up: String,
    #[serde(default = "default_direction_down")]
    pub direction_down: String,
}

fn default_layout() -> String {
    "qwerty".to_string()
}

fn default_direction_up() -> String {
    "k".to_string()
}

fn default_direction_down() -> String {
    "j".to_string()
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            layout: default_layout(),
            direction_up: default_direction_up(),
            direction_down: default_direction_down(),
        }
    }
}

impl KeyboardConfig {
    pub fn colemak() -> Self {
        Self {
            layout: "colemak".to_string(),
            direction_up: "u".to_string(),
            direction_down: "e".to_string(),
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
    #[serde(default = "default_notes_app")]
    pub app: NotesApp,
    pub folder: Option<String>,
    pub vault: Option<String>,
}

fn default_notes_app() -> NotesApp {
    NotesApp::AppleNotes
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

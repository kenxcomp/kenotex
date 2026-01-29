use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Processing,
    Search,
}

impl AppMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Insert => "INSERT",
            AppMode::Visual => "VISUAL",
            AppMode::Processing => "PROCESSING",
            AppMode::Search => "SEARCH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum View {
    #[default]
    Editor,
    DraftList,
    ArchiveList,
}

impl View {
    pub fn as_str(&self) -> &'static str {
        match self {
            View::Editor => "Editor",
            View::DraftList => "Drafts",
            View::ArchiveList => "Archive",
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::molecules::editor::VisualType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum AppMode {
    #[default]
    Normal,
    Insert,
    Visual(VisualType),
    Processing,
    Search,
    ConfirmDelete,
}


impl AppMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Insert => "INSERT",
            AppMode::Visual(VisualType::Character) => "VISUAL",
            AppMode::Visual(VisualType::Line) => "VISUAL LINE",
            AppMode::Visual(VisualType::Block) => "VISUAL BLOCK",
            AppMode::Processing => "PROCESSING",
            AppMode::Search => "SEARCH",
            AppMode::ConfirmDelete => "CONFIRM",
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

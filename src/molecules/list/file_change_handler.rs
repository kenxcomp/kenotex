use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::atoms::storage::file_watcher::FileEvent;

const SUPPRESSION_WINDOW_MS: u64 = 2000;

#[derive(Debug)]
pub enum FileChangeAction {
    ReloadNote { id: String, is_archived: bool },
    NewNote { id: String, is_archived: bool },
    DeletedNote { id: String, is_archived: bool },
    Suppressed,
}

pub struct FileChangeTracker {
    save_timestamps: HashMap<String, Instant>,
}

impl FileChangeTracker {
    pub fn new() -> Self {
        Self {
            save_timestamps: HashMap::new(),
        }
    }

    pub fn record_save(&mut self, note_id: &str) {
        self.save_timestamps
            .insert(note_id.to_string(), Instant::now());
    }

    fn should_suppress(&self, note_id: &str) -> bool {
        if let Some(timestamp) = self.save_timestamps.get(note_id) {
            timestamp.elapsed() < Duration::from_millis(SUPPRESSION_WINDOW_MS)
        } else {
            false
        }
    }

    pub fn cleanup(&mut self) {
        let cutoff = Duration::from_secs(10);
        self.save_timestamps
            .retain(|_, ts| ts.elapsed() < cutoff);
    }
}

fn extract_note_id(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
}

pub fn classify_event(
    event: &FileEvent,
    tracker: &FileChangeTracker,
    known_ids: &[String],
) -> FileChangeAction {
    match event {
        FileEvent::Modified(path, is_archived) => {
            if let Some(id) = extract_note_id(path) {
                if tracker.should_suppress(&id) {
                    return FileChangeAction::Suppressed;
                }
                if known_ids.contains(&id) {
                    FileChangeAction::ReloadNote {
                        id,
                        is_archived: *is_archived,
                    }
                } else {
                    FileChangeAction::NewNote {
                        id,
                        is_archived: *is_archived,
                    }
                }
            } else {
                FileChangeAction::Suppressed
            }
        }
        FileEvent::Removed(path, is_archived) => {
            if let Some(id) = extract_note_id(path) {
                if tracker.should_suppress(&id) {
                    return FileChangeAction::Suppressed;
                }
                FileChangeAction::DeletedNote {
                    id,
                    is_archived: *is_archived,
                }
            } else {
                FileChangeAction::Suppressed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_suppress_recent_save() {
        let mut tracker = FileChangeTracker::new();
        tracker.record_save("note-123");

        let event = FileEvent::Modified(PathBuf::from("/tmp/drafts/note-123.md"), false);
        let action = classify_event(&event, &tracker, &["note-123".to_string()]);
        assert!(matches!(action, FileChangeAction::Suppressed));
    }

    #[test]
    fn test_allow_after_suppression_window() {
        let mut tracker = FileChangeTracker::new();
        tracker.record_save("note-123");

        // Simulate time passage beyond suppression window (2s)
        tracker
            .save_timestamps
            .insert("note-123".to_string(), Instant::now() - Duration::from_secs(3));

        let event = FileEvent::Modified(PathBuf::from("/tmp/drafts/note-123.md"), false);
        let action = classify_event(&event, &tracker, &["note-123".to_string()]);
        assert!(matches!(action, FileChangeAction::ReloadNote { .. }));
    }

    #[test]
    fn test_new_note_detection() {
        let tracker = FileChangeTracker::new();
        let event = FileEvent::Modified(PathBuf::from("/tmp/drafts/new-note.md"), false);
        let action = classify_event(&event, &tracker, &["existing-note".to_string()]);
        assert!(matches!(action, FileChangeAction::NewNote { .. }));
    }

    #[test]
    fn test_deleted_note_detection() {
        let tracker = FileChangeTracker::new();
        let event = FileEvent::Removed(PathBuf::from("/tmp/drafts/old-note.md"), false);
        let action = classify_event(&event, &tracker, &["old-note".to_string()]);
        assert!(matches!(action, FileChangeAction::DeletedNote { .. }));
    }

    #[test]
    fn test_cleanup_old_entries() {
        let mut tracker = FileChangeTracker::new();
        tracker
            .save_timestamps
            .insert("old".to_string(), Instant::now() - Duration::from_secs(20));
        tracker.record_save("recent");

        tracker.cleanup();
        assert!(!tracker.save_timestamps.contains_key("old"));
        assert!(tracker.save_timestamps.contains_key("recent"));
    }
}

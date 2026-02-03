use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use notify::RecursiveMode;

#[derive(Debug, Clone)]
pub enum FileEvent {
    Modified(PathBuf, bool),
    Removed(PathBuf, bool),
}

pub struct FileWatcherHandle {
    _debouncer: Debouncer<notify::RecommendedWatcher>,
    pub receiver: mpsc::Receiver<FileEvent>,
}

pub fn start_watcher(
    drafts_dir: &Path,
    archives_dir: &Path,
    debounce_ms: u64,
) -> Result<FileWatcherHandle> {
    let (tx, rx) = mpsc::channel();

    let archives_dir_owned = archives_dir.to_path_buf();

    let mut debouncer = new_debouncer(
        Duration::from_millis(debounce_ms),
        move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = res {
                for event in events {
                    let path = &event.path;
                    if path.extension().is_some_and(|ext| ext == "md") {
                        let is_archived = path.starts_with(&archives_dir_owned);
                        let file_event = match event.kind {
                            DebouncedEventKind::Any => {
                                if path.exists() {
                                    FileEvent::Modified(path.clone(), is_archived)
                                } else {
                                    FileEvent::Removed(path.clone(), is_archived)
                                }
                            }
                            DebouncedEventKind::AnyContinuous | _ => continue,
                        };
                        let _ = tx.send(file_event);
                    }
                }
            }
        },
    )
    .with_context(|| "Failed to create file watcher")?;

    if drafts_dir.exists() {
        debouncer
            .watcher()
            .watch(drafts_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("Failed to watch drafts dir: {:?}", drafts_dir))?;
    }

    if archives_dir.exists() {
        debouncer
            .watcher()
            .watch(archives_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("Failed to watch archives dir: {:?}", archives_dir))?;
    }

    Ok(FileWatcherHandle {
        _debouncer: debouncer,
        receiver: rx,
    })
}

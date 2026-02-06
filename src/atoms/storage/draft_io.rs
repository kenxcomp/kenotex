use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::Note;

fn drafts_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("drafts")
}

fn archives_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("archives")
}

pub fn ensure_data_dirs(base_dir: &Path) -> Result<()> {
    let dir = drafts_dir(base_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create drafts directory: {:?}", dir))?;
    }
    let archive_dir = archives_dir(base_dir);
    if !archive_dir.exists() {
        fs::create_dir_all(&archive_dir)
            .with_context(|| format!("Failed to create archives directory: {:?}", archive_dir))?;
    }
    Ok(())
}

fn draft_path(base_dir: &Path, id: &str, is_archived: bool) -> PathBuf {
    let dir = if is_archived {
        archives_dir(base_dir)
    } else {
        drafts_dir(base_dir)
    };
    dir.join(format!("{}.md", id))
}

pub fn load_draft(base_dir: &Path, id: &str, is_archived: bool) -> Result<Note> {
    let path = draft_path(base_dir, id, is_archived);

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read draft: {:?}", path))?;

    let metadata = fs::metadata(&path)?;
    let created_at: DateTime<Utc> = metadata
        .created()
        .map(|t| t.into())
        .unwrap_or_else(|_| Utc::now());
    let updated_at: DateTime<Utc> = metadata.modified().map(|t| t.into()).unwrap_or(created_at);

    let title = Note::extract_title(&content);

    Ok(Note {
        id: id.to_string(),
        title,
        content,
        created_at,
        updated_at,
        is_archived,
        selected: false,
    })
}

pub fn load_all_drafts(base_dir: &Path, archived: bool) -> Result<Vec<Note>> {
    let dir = if archived {
        archives_dir(base_dir)
    } else {
        drafts_dir(base_dir)
    };

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md")
            && let Some(stem) = path.file_stem() {
                let id = stem.to_string_lossy().to_string();
                match load_draft(base_dir, &id, archived) {
                    Ok(note) => notes.push(note),
                    Err(e) => eprintln!("Warning: Failed to load draft {}: {}", id, e),
                }
            }
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

pub fn save_draft(base_dir: &Path, note: &Note) -> Result<()> {
    ensure_data_dirs(base_dir)?;
    let path = draft_path(base_dir, &note.id, note.is_archived);

    fs::write(&path, &note.content).with_context(|| format!("Failed to save draft: {:?}", path))?;

    Ok(())
}

pub fn delete_draft(base_dir: &Path, id: &str, is_archived: bool) -> Result<()> {
    let path = draft_path(base_dir, id, is_archived);

    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to delete draft: {:?}", path))?;
    }

    Ok(())
}

pub fn archive_draft(base_dir: &Path, note: &mut Note) -> Result<()> {
    let old_path = draft_path(base_dir, &note.id, false);
    note.is_archived = true;
    let new_path = draft_path(base_dir, &note.id, true);

    ensure_data_dirs(base_dir)?;

    if old_path.exists() {
        fs::rename(&old_path, &new_path)
            .with_context(|| format!("Failed to archive draft: {:?}", old_path))?;
    } else {
        save_draft(base_dir, note)?;
    }

    Ok(())
}

pub fn restore_draft(base_dir: &Path, note: &mut Note) -> Result<()> {
    let old_path = draft_path(base_dir, &note.id, true);
    note.is_archived = false;
    let new_path = draft_path(base_dir, &note.id, false);

    ensure_data_dirs(base_dir)?;

    if old_path.exists() {
        fs::rename(&old_path, &new_path)
            .with_context(|| format!("Failed to restore draft: {:?}", old_path))?;
    } else {
        save_draft(base_dir, note)?;
    }

    Ok(())
}

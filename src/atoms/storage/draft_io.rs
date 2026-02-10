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
            && let Some(stem) = path.file_stem()
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_base_dir() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("kenotex_test_{}", ts));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn make_note(id: &str, content: &str) -> Note {
        Note::new(
            id.to_string(),
            Note::extract_title(content),
            content.to_string(),
        )
    }

    #[test]
    fn test_ensure_data_dirs() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        assert!(base.join("drafts").exists());
        assert!(base.join("archives").exists());
        cleanup(&base);
    }

    #[test]
    fn test_ensure_data_dirs_idempotent() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        ensure_data_dirs(&base).unwrap();
        assert!(base.join("drafts").exists());
        cleanup(&base);
    }

    #[test]
    fn test_save_and_load_draft() {
        let base = temp_base_dir();
        let note = make_note("note1", "# Hello\nWorld");
        save_draft(&base, &note).unwrap();

        let loaded = load_draft(&base, "note1", false).unwrap();
        assert_eq!(loaded.id, "note1");
        assert_eq!(loaded.content, "# Hello\nWorld");
        assert_eq!(loaded.title, "Hello");
        assert!(!loaded.is_archived);
        cleanup(&base);
    }

    #[test]
    fn test_load_draft_not_found() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        let result = load_draft(&base, "nonexistent", false);
        assert!(result.is_err());
        cleanup(&base);
    }

    #[test]
    fn test_save_and_delete_draft() {
        let base = temp_base_dir();
        let note = make_note("note2", "content");
        save_draft(&base, &note).unwrap();
        assert!(base.join("drafts/note2.md").exists());

        delete_draft(&base, "note2", false).unwrap();
        assert!(!base.join("drafts/note2.md").exists());
        cleanup(&base);
    }

    #[test]
    fn test_delete_nonexistent_draft() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        // Should not error
        delete_draft(&base, "nonexistent", false).unwrap();
        cleanup(&base);
    }

    #[test]
    fn test_load_all_drafts_empty() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        let drafts = load_all_drafts(&base, false).unwrap();
        assert!(drafts.is_empty());
        cleanup(&base);
    }

    #[test]
    fn test_load_all_drafts_nonexistent_dir() {
        let base = temp_base_dir();
        // Don't create dirs
        let drafts = load_all_drafts(&base, false).unwrap();
        assert!(drafts.is_empty());
        cleanup(&base);
    }

    #[test]
    fn test_load_all_drafts_multiple() {
        let base = temp_base_dir();
        save_draft(&base, &make_note("a", "First")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        save_draft(&base, &make_note("b", "Second")).unwrap();

        let drafts = load_all_drafts(&base, false).unwrap();
        assert_eq!(drafts.len(), 2);
        // Should be sorted by updated_at descending
        assert_eq!(drafts[0].id, "b");
        assert_eq!(drafts[1].id, "a");
        cleanup(&base);
    }

    #[test]
    fn test_archive_draft() {
        let base = temp_base_dir();
        let mut note = make_note("note3", "to archive");
        save_draft(&base, &note).unwrap();
        assert!(base.join("drafts/note3.md").exists());

        archive_draft(&base, &mut note).unwrap();
        assert!(note.is_archived);
        assert!(!base.join("drafts/note3.md").exists());
        assert!(base.join("archives/note3.md").exists());
        cleanup(&base);
    }

    #[test]
    fn test_restore_draft() {
        let base = temp_base_dir();
        let mut note = make_note("note4", "archived content");
        note.is_archived = true;
        save_draft(&base, &note).unwrap();
        assert!(base.join("archives/note4.md").exists());

        restore_draft(&base, &mut note).unwrap();
        assert!(!note.is_archived);
        assert!(!base.join("archives/note4.md").exists());
        assert!(base.join("drafts/note4.md").exists());
        cleanup(&base);
    }

    #[test]
    fn test_archive_and_restore_roundtrip() {
        let base = temp_base_dir();
        let mut note = make_note("note5", "roundtrip");
        save_draft(&base, &note).unwrap();

        archive_draft(&base, &mut note).unwrap();
        assert!(note.is_archived);

        restore_draft(&base, &mut note).unwrap();
        assert!(!note.is_archived);

        let loaded = load_draft(&base, "note5", false).unwrap();
        assert_eq!(loaded.content, "roundtrip");
        cleanup(&base);
    }

    #[test]
    fn test_load_all_archived() {
        let base = temp_base_dir();
        let mut note = make_note("arch1", "archived");
        note.is_archived = true;
        save_draft(&base, &note).unwrap();

        let archives = load_all_drafts(&base, true).unwrap();
        assert_eq!(archives.len(), 1);
        assert_eq!(archives[0].id, "arch1");
        cleanup(&base);
    }

    #[test]
    fn test_save_draft_cjk_content() {
        let base = temp_base_dir();
        let note = make_note("cjk1", "# 你好世界\n这是内容");
        save_draft(&base, &note).unwrap();

        let loaded = load_draft(&base, "cjk1", false).unwrap();
        assert_eq!(loaded.content, "# 你好世界\n这是内容");
        assert_eq!(loaded.title, "你好世界");
        cleanup(&base);
    }

    #[test]
    fn test_archive_nonexistent_file_saves_instead() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        let mut note = make_note("nosource", "content");
        // Archive without saving first - should save to archive
        archive_draft(&base, &mut note).unwrap();
        assert!(note.is_archived);
        assert!(base.join("archives/nosource.md").exists());
        cleanup(&base);
    }

    #[test]
    fn test_restore_nonexistent_file_saves_instead() {
        let base = temp_base_dir();
        ensure_data_dirs(&base).unwrap();
        let mut note = make_note("nosource2", "content");
        note.is_archived = true;
        // Restore without an archive file - should save to drafts
        restore_draft(&base, &mut note).unwrap();
        assert!(!note.is_archived);
        assert!(base.join("drafts/nosource2.md").exists());
        cleanup(&base);
    }
}

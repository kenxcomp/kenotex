use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// Resolves the editor command from environment variables.
/// Checks `$VISUAL`, then `$EDITOR`, falling back to `"vi"`.
pub fn resolve_editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string())
}

/// Writes content to a temporary file for external editing.
/// Returns the path to the created temp file.
pub fn write_temp_file(content: &str) -> Result<PathBuf> {
    let tmp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let path = tmp_dir.join(format!("kenotex_{}.md", pid));
    fs::write(&path, content).context("Failed to write temp file for external editor")?;
    Ok(path)
}

/// Spawns the editor process and waits for it to exit.
/// The editor string is split by whitespace to support commands like `"code --wait"`.
/// Returns `true` if the editor exited successfully.
pub fn spawn_editor(editor: &str, path: &Path) -> Result<bool> {
    let mut parts = editor.split_whitespace();
    let program = parts.next().context("Empty editor command")?;
    let args: Vec<&str> = parts.collect();

    let status = Command::new(program)
        .args(&args)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to spawn editor: {}", editor))?;

    Ok(status.success())
}

/// Reads the content back from the temp file after editing.
pub fn read_temp_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).context("Failed to read temp file after external editor")
}

/// Removes the temp file, ignoring any errors.
pub fn cleanup_temp_file(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to save and restore env vars around test code.
    /// Uses unsafe blocks required by Rust 2024 edition.
    unsafe fn with_env_vars<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
        // Save originals
        let originals: Vec<(&str, Option<String>)> = vars
            .iter()
            .map(|(k, _)| (*k, std::env::var(k).ok()))
            .collect();

        // Set test values
        for (key, val) in vars {
            match val {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }

        f();

        // Restore originals
        for (key, orig) in &originals {
            match orig {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }

    #[test]
    fn test_resolve_editor_visual() {
        unsafe {
            with_env_vars(&[("VISUAL", Some("nvim")), ("EDITOR", Some("vim"))], || {
                assert_eq!(resolve_editor(), "nvim")
            });
        }
    }

    #[test]
    fn test_resolve_editor_editor_fallback() {
        unsafe {
            with_env_vars(&[("VISUAL", None), ("EDITOR", Some("nano"))], || {
                assert_eq!(resolve_editor(), "nano")
            });
        }
    }

    #[test]
    fn test_resolve_editor_vi_fallback() {
        unsafe {
            with_env_vars(&[("VISUAL", None), ("EDITOR", None)], || {
                assert_eq!(resolve_editor(), "vi")
            });
        }
    }

    #[test]
    fn test_write_and_read_temp_file() {
        let content = "Hello, external editor!\nLine two.";
        let path = write_temp_file(content).unwrap();
        assert!(path.exists());

        let read_back = read_temp_file(&path).unwrap();
        assert_eq!(read_back, content);

        cleanup_temp_file(&path);
        assert!(!path.exists());
    }

    #[test]
    fn test_cleanup_nonexistent_file() {
        let path = std::env::temp_dir().join("kenotex_nonexistent_test.md");
        // Should not panic
        cleanup_temp_file(&path);
    }
}

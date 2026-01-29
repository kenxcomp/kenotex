use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::types::Config;

pub fn config_dir() -> PathBuf {
    // Prefer ~/.config/kenotex on Unix-like systems for better compatibility
    // with dotfiles management tools
    if let Some(home) = dirs::home_dir() {
        let xdg_config = home.join(".config").join("kenotex");
        if xdg_config.exists() || cfg!(unix) {
            return xdg_config;
        }
    }

    // Fallback to system default (e.g., ~/Library/Application Support on macOS)
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kenotex")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn ensure_config_dir() -> Result<PathBuf> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {:?}", dir))?;
    }
    Ok(dir)
}

pub fn load_config() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read config: {:?}", path))?;

    let config: Config =
        toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    ensure_config_dir()?;
    let path = config_path();

    let content = toml::to_string_pretty(config).with_context(|| "Failed to serialize config")?;

    fs::write(&path, content).with_context(|| format!("Failed to write config: {:?}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.theme, "tokyo_night");
        assert_eq!(config.general.leader_key, " ");
        assert_eq!(config.keyboard.layout, "qwerty");
    }

    #[test]
    fn test_config_path_xdg() {
        // Verify config path uses ~/.config/kenotex on Unix
        let path = config_path();
        if cfg!(unix) {
            assert!(path.to_string_lossy().contains(".config/kenotex"));
        }
    }
}

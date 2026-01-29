use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::types::Config;

pub fn config_dir() -> PathBuf {
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
}

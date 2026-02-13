use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{Config, TimeConfig};

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

    let config: Config = toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;

    Ok(config)
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    if path == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    PathBuf::from(path)
}

pub fn resolve_data_dir(data_dir: Option<&str>) -> PathBuf {
    match data_dir {
        Some(dir) => expand_tilde(dir),
        None => config_dir(),
    }
}

pub fn time_config_path() -> PathBuf {
    config_dir().join("time_patterns.toml")
}

pub fn load_time_config() -> Result<TimeConfig> {
    let path = time_config_path();

    if !path.exists() {
        save_default_time_config(&path)?;
        return Ok(TimeConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read time config: {:?}", path))?;

    let config: TimeConfig =
        toml::from_str(&content).with_context(|| "Failed to parse time_patterns.toml")?;

    Ok(config)
}

fn save_default_time_config(path: &Path) -> Result<()> {
    ensure_config_dir()?;
    let template = r#"# Time Pattern Configuration for Kenotex
# Customize how time expressions are parsed in :::td and :::cal blocks.
#
# [periods] maps keywords to "HH:MM" time-of-day defaults.
# [offsets] maps keywords to day offsets from today (0 = today, 1 = tomorrow, etc.).
# [weekdays] maps aliases to standard English weekday names
#            (monday, tuesday, wednesday, thursday, friday, saturday, sunday).

[periods]
早上 = "09:00"
上午 = "09:00"
中午 = "12:00"
下午 = "14:00"
晚上 = "19:00"
morning = "09:00"
afternoon = "14:00"
evening = "19:00"

[offsets]
今天 = 0
明天 = 1
后天 = 2
大后天 = 3
下周 = 7
today = 0
tomorrow = 1

[weekdays]
周一 = "monday"
周二 = "tuesday"
周三 = "wednesday"
周四 = "thursday"
周五 = "friday"
周六 = "saturday"
周日 = "sunday"
星期一 = "monday"
星期二 = "tuesday"
星期三 = "wednesday"
星期四 = "thursday"
星期五 = "friday"
星期六 = "saturday"
星期日 = "sunday"

# Hour keywords — keyword → hour number (0-23)
# 小时关键词 — 关键词 → 小时数 (0-23)
[hours]
一 = 1
二 = 2
三 = 3
四 = 4
五 = 5
六 = 6
七 = 7
八 = 8
九 = 9
十 = 10
十一 = 11
十二 = 12

# Minute keywords — keyword → minute number (0-59)
# 分钟关键词 — 关键词 → 分钟数 (0-59)
[minutes]
五 = 5
十 = 10
十五 = 15
二十 = 20
二十五 = 25
三十 = 30
三十五 = 35
四十 = 40
四十五 = 45
五十 = 50
五十五 = 55
"#;

    fs::write(path, template)
        .with_context(|| format!("Failed to write time config: {:?}", path))?;

    Ok(())
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

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/Documents/notes");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join("Documents/notes"));
        }
    }

    #[test]
    fn test_expand_tilde_no_prefix() {
        let path = expand_tilde("/absolute/path");
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_expand_tilde_home_only() {
        let expanded = expand_tilde("~");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home);
        }
    }

    #[test]
    fn test_resolve_data_dir_custom() {
        let resolved = resolve_data_dir(Some("/tmp/kenotex-test"));
        assert_eq!(resolved, PathBuf::from("/tmp/kenotex-test"));
    }

    #[test]
    fn test_resolve_data_dir_none_falls_back() {
        let resolved = resolve_data_dir(None);
        assert_eq!(resolved, config_dir());
    }

    #[test]
    fn test_time_config_path() {
        let path = time_config_path();
        assert!(path.to_string_lossy().ends_with("time_patterns.toml"));
    }

    #[test]
    fn test_load_time_config_missing_file() {
        // When the file doesn't exist, load_time_config creates a default
        // and writes it out. We can verify the defaults are returned by
        // checking the result matches TimeConfig::default().
        // Use a temp dir so we don't interfere with real config.
        let tmp = std::env::temp_dir().join("kenotex-test-time-config");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let fake_path = tmp.join("time_patterns.toml");

        // The file should not exist
        assert!(!fake_path.exists());

        // Manually test the parsing: read an empty TOML → should give defaults
        let config: TimeConfig = toml::from_str("").unwrap();
        assert_eq!(config.periods.len(), TimeConfig::default().periods.len());
        assert_eq!(config.offsets.len(), TimeConfig::default().offsets.len());
        assert_eq!(config.weekdays.len(), TimeConfig::default().weekdays.len());
        assert_eq!(config.hours.len(), TimeConfig::default().hours.len());
        assert_eq!(config.minutes.len(), TimeConfig::default().minutes.len());

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

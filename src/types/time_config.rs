use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    #[serde(default = "default_periods")]
    pub periods: HashMap<String, String>,
    #[serde(default = "default_offsets")]
    pub offsets: HashMap<String, i64>,
    #[serde(default = "default_weekdays")]
    pub weekdays: HashMap<String, String>,
    #[serde(default = "default_hours")]
    pub hours: HashMap<String, u32>,
    #[serde(default = "default_minutes")]
    pub minutes: HashMap<String, u32>,
}

fn default_periods() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("早上".to_string(), "09:00".to_string());
    m.insert("上午".to_string(), "09:00".to_string());
    m.insert("中午".to_string(), "12:00".to_string());
    m.insert("下午".to_string(), "14:00".to_string());
    m.insert("晚上".to_string(), "19:00".to_string());
    m.insert("morning".to_string(), "09:00".to_string());
    m.insert("afternoon".to_string(), "14:00".to_string());
    m.insert("evening".to_string(), "19:00".to_string());
    m
}

fn default_offsets() -> HashMap<String, i64> {
    let mut m = HashMap::new();
    m.insert("今天".to_string(), 0);
    m.insert("明天".to_string(), 1);
    m.insert("后天".to_string(), 2);
    m.insert("大后天".to_string(), 3);
    m.insert("下周".to_string(), 7);
    m.insert("today".to_string(), 0);
    m.insert("tomorrow".to_string(), 1);
    m
}

fn default_weekdays() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("周一".to_string(), "monday".to_string());
    m.insert("周二".to_string(), "tuesday".to_string());
    m.insert("周三".to_string(), "wednesday".to_string());
    m.insert("周四".to_string(), "thursday".to_string());
    m.insert("周五".to_string(), "friday".to_string());
    m.insert("周六".to_string(), "saturday".to_string());
    m.insert("周日".to_string(), "sunday".to_string());
    m.insert("星期一".to_string(), "monday".to_string());
    m.insert("星期二".to_string(), "tuesday".to_string());
    m.insert("星期三".to_string(), "wednesday".to_string());
    m.insert("星期四".to_string(), "thursday".to_string());
    m.insert("星期五".to_string(), "friday".to_string());
    m.insert("星期六".to_string(), "saturday".to_string());
    m.insert("星期日".to_string(), "sunday".to_string());
    m
}

fn default_hours() -> HashMap<String, u32> {
    let mut m = HashMap::new();
    m.insert("一".to_string(), 1);
    m.insert("二".to_string(), 2);
    m.insert("三".to_string(), 3);
    m.insert("四".to_string(), 4);
    m.insert("五".to_string(), 5);
    m.insert("六".to_string(), 6);
    m.insert("七".to_string(), 7);
    m.insert("八".to_string(), 8);
    m.insert("九".to_string(), 9);
    m.insert("十".to_string(), 10);
    m.insert("十一".to_string(), 11);
    m.insert("十二".to_string(), 12);
    m
}

fn default_minutes() -> HashMap<String, u32> {
    let mut m = HashMap::new();
    m.insert("五".to_string(), 5);
    m.insert("十".to_string(), 10);
    m.insert("十五".to_string(), 15);
    m.insert("二十".to_string(), 20);
    m.insert("二十五".to_string(), 25);
    m.insert("三十".to_string(), 30);
    m.insert("三十五".to_string(), 35);
    m.insert("四十".to_string(), 40);
    m.insert("四十五".to_string(), 45);
    m.insert("五十".to_string(), 50);
    m.insert("五十五".to_string(), 55);
    m
}

impl TimeConfig {
    /// Parse a period value ("HH:MM") into (hour, minute).
    pub fn parse_period(&self, key: &str) -> Option<(u32, u32)> {
        let value = self.periods.get(key)?;
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let hour: u32 = parts[0].parse().ok()?;
        let minute: u32 = parts[1].parse().ok()?;
        Some((hour, minute))
    }

    /// Collect all keywords from periods/offsets/weekdays/hours/minutes + hardcoded "am"/"pm"
    /// + standard weekday names (monday, tuesday, etc.).
    pub fn all_keywords(&self) -> Vec<String> {
        let mut keywords: Vec<String> = Vec::new();
        keywords.extend(self.periods.keys().cloned());
        keywords.extend(self.offsets.keys().cloned());
        keywords.extend(self.weekdays.keys().cloned());
        for key in self.hours.keys() {
            keywords.push(format!("{}点", key));
        }
        for key in self.minutes.keys() {
            keywords.push(format!("{}分", key));
        }
        keywords.push("am".to_string());
        keywords.push("pm".to_string());
        for name in [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ] {
            if !keywords.iter().any(|k| k == name) {
                keywords.push(name.to_string());
            }
        }
        keywords
    }
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            periods: default_periods(),
            offsets: default_offsets(),
            weekdays: default_weekdays(),
            hours: default_hours(),
            minutes: default_minutes(),
        }
    }
}

impl PartialEq for TimeConfig {
    fn eq(&self, other: &Self) -> bool {
        self.periods == other.periods
            && self.offsets == other.offsets
            && self.weekdays == other.weekdays
            && self.hours == other.hours
            && self.minutes == other.minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_all_periods() {
        let config = TimeConfig::default();
        let expected = [
            "早上",
            "上午",
            "中午",
            "下午",
            "晚上",
            "morning",
            "afternoon",
            "evening",
        ];
        for key in expected {
            assert!(
                config.periods.contains_key(key),
                "missing period key: {}",
                key
            );
        }
        assert_eq!(config.periods.len(), expected.len());
    }

    #[test]
    fn test_default_has_all_offsets() {
        let config = TimeConfig::default();
        assert_eq!(config.offsets["今天"], 0);
        assert_eq!(config.offsets["明天"], 1);
        assert_eq!(config.offsets["后天"], 2);
        assert_eq!(config.offsets["大后天"], 3);
        assert_eq!(config.offsets["下周"], 7);
        assert_eq!(config.offsets["today"], 0);
        assert_eq!(config.offsets["tomorrow"], 1);
        assert_eq!(config.offsets.len(), 7);
    }

    #[test]
    fn test_default_has_weekdays() {
        let config = TimeConfig::default();
        // 周一~周日
        for day in ["周一", "周二", "周三", "周四", "周五", "周六", "周日"] {
            assert!(
                config.weekdays.contains_key(day),
                "missing weekday key: {}",
                day
            );
        }
        // 星期一~星期日
        for day in [
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
            "星期日",
        ] {
            assert!(
                config.weekdays.contains_key(day),
                "missing weekday key: {}",
                day
            );
        }
        assert_eq!(config.weekdays.len(), 14);
    }

    #[test]
    fn test_parse_period_valid() {
        let mut config = TimeConfig::default();
        // Test existing period
        assert_eq!(config.parse_period("早上"), Some((9, 0)));
        assert_eq!(config.parse_period("下午"), Some((14, 0)));
        // Insert a custom period with non-zero minutes
        config
            .periods
            .insert("test".to_string(), "14:30".to_string());
        assert_eq!(config.parse_period("test"), Some((14, 30)));
    }

    #[test]
    fn test_parse_period_invalid() {
        let mut config = TimeConfig::default();
        // Non-existent key
        assert_eq!(config.parse_period("nonexistent"), None);
        // Insert invalid format values
        config
            .periods
            .insert("bad1".to_string(), "invalid".to_string());
        assert_eq!(config.parse_period("bad1"), None);
        config
            .periods
            .insert("bad2".to_string(), "25:00".to_string());
        // "25:00" parses as (25, 0) — parse_period doesn't validate range
        // It just parses the numbers, so 25:00 returns Some((25, 0))
        assert_eq!(config.parse_period("bad2"), Some((25, 0)));
    }

    #[test]
    fn test_all_keywords_contains_essentials() {
        let config = TimeConfig::default();
        let keywords = config.all_keywords();
        for expected in ["明天", "tomorrow", "am", "pm", "monday"] {
            assert!(
                keywords.iter().any(|k| k == expected),
                "missing keyword: {}",
                expected
            );
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = TimeConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: TimeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        // Only specify [periods], offsets and weekdays should use defaults
        let toml_str = "
[periods]
\"早上\" = \"07:00\"
";
        let config: TimeConfig = toml::from_str(toml_str).unwrap();
        // Periods should only have what we specified
        assert_eq!(config.periods.len(), 1);
        assert_eq!(config.periods["早上"], "07:00");
        // Offsets and weekdays should be defaults
        assert_eq!(config.offsets["明天"], 1);
        assert_eq!(config.offsets["tomorrow"], 1);
        assert_eq!(config.weekdays["周一"], "monday");
        assert_eq!(config.weekdays["星期日"], "sunday");
    }

    #[test]
    fn test_default_has_all_hours() {
        let config = TimeConfig::default();
        let expected: Vec<(&str, u32)> = vec![
            ("一", 1),
            ("二", 2),
            ("三", 3),
            ("四", 4),
            ("五", 5),
            ("六", 6),
            ("七", 7),
            ("八", 8),
            ("九", 9),
            ("十", 10),
            ("十一", 11),
            ("十二", 12),
        ];
        for (key, val) in &expected {
            assert_eq!(
                config.hours.get(*key),
                Some(val),
                "missing or wrong hour: {} → {}",
                key,
                val
            );
        }
        assert_eq!(config.hours.len(), expected.len());
    }

    #[test]
    fn test_default_has_all_minutes() {
        let config = TimeConfig::default();
        let expected: Vec<(&str, u32)> = vec![
            ("五", 5),
            ("十", 10),
            ("十五", 15),
            ("二十", 20),
            ("二十五", 25),
            ("三十", 30),
            ("三十五", 35),
            ("四十", 40),
            ("四十五", 45),
            ("五十", 50),
            ("五十五", 55),
        ];
        for (key, val) in &expected {
            assert_eq!(
                config.minutes.get(*key),
                Some(val),
                "missing or wrong minute: {} → {}",
                key,
                val
            );
        }
        assert_eq!(config.minutes.len(), expected.len());
    }

    #[test]
    fn test_all_keywords_contains_hour_minute() {
        let config = TimeConfig::default();
        let keywords = config.all_keywords();
        // Hour key "七" should produce "七点" in keywords
        assert!(
            keywords.iter().any(|k| k == "七点"),
            "all_keywords() should contain '七点'"
        );
        // Minute key "三十" should produce "三十分" in keywords
        assert!(
            keywords.iter().any(|k| k == "三十分"),
            "all_keywords() should contain '三十分'"
        );
    }

    #[test]
    fn test_serde_roundtrip_with_hours_minutes() {
        let config = TimeConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: TimeConfig = toml::from_str(&toml_str).unwrap();
        // Verify hours and minutes survive roundtrip
        assert_eq!(config.hours, parsed.hours);
        assert_eq!(config.minutes, parsed.minutes);
    }

    #[test]
    fn test_partial_toml_uses_default_hours_minutes() {
        // Only specify [periods], hours and minutes should use defaults
        let toml_str = r#"
[periods]
"早上" = "07:00"
"#;
        let config: TimeConfig = toml::from_str(toml_str).unwrap();
        // Periods should only have what we specified
        assert_eq!(config.periods.len(), 1);
        // Hours and minutes should be defaults
        assert_eq!(config.hours.get("七"), Some(&7u32));
        assert_eq!(config.minutes.get("三十"), Some(&30u32));
        assert_eq!(config.hours.len(), 12);
        assert_eq!(config.minutes.len(), 11);
    }
}

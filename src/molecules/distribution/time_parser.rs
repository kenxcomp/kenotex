use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc, Weekday};
use chrono_english::{Dialect, parse_date_string};
use regex::Regex;

/// Parse @-prefixed time expressions.
/// Examples: "@明天早上8点", "@9pm", "@tomorrow", "@下周一"
pub fn parse_at_time_expression(text: &str) -> Option<DateTime<Utc>> {
    // Extract time portion after @ symbol
    let time_expr = extract_at_time(text)?;

    // Validate: must look like a time expression before attempting to parse
    if !is_valid_time_keyword(&time_expr) {
        return None;
    }

    // Try parsing as regular time expression (reuse existing logic)
    parse_time_expression(&time_expr)
}

/// Extracts the time portion after @ symbol.
/// Returns the substring after @ until whitespace or end of string.
fn extract_at_time(text: &str) -> Option<String> {
    let at_pos = text.find('@')?;
    let remaining = &text[at_pos + 1..];

    // Extract until whitespace, punctuation (excluding Chinese chars and colon), or end
    let mut end_pos = 0;
    for (i, ch) in remaining.char_indices() {
        if ch.is_whitespace() {
            break;
        }
        // Stop at ASCII punctuation except colon (for time like "3:30")
        if ch.is_ascii_punctuation() && ch != ':' {
            break;
        }
        end_pos = i + ch.len_utf8();
    }

    if end_pos == 0 && !remaining.is_empty() {
        end_pos = remaining.len();
    }

    if end_pos == 0 {
        return None;
    }

    Some(remaining[..end_pos].to_string())
}

/// Check if text looks like a valid time keyword.
/// Simple heuristic: contains digits OR known time-related keywords.
fn is_valid_time_keyword(text: &str) -> bool {
    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    let keywords = [
        "明天",
        "今天",
        "后天",
        "下周",
        "周",
        "早上",
        "上午",
        "下午",
        "晚上",
        "中午",
        "tomorrow",
        "today",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "morning",
        "evening",
        "afternoon",
        "am",
        "pm",
    ];
    let has_keyword = keywords.iter().any(|kw| text.contains(kw));

    has_digit || has_keyword
}

pub fn parse_time_expression(text: &str) -> Option<DateTime<Utc>> {
    if let Some(dt) = parse_english_time(text) {
        return Some(dt);
    }

    if let Some(dt) = parse_chinese_time(text) {
        return Some(dt);
    }

    None
}

fn parse_english_time(text: &str) -> Option<DateTime<Utc>> {
    let now = Local::now();

    if let Ok(dt) = parse_date_string(text, now, Dialect::Us) {
        return Some(dt.with_timezone(&Utc));
    }

    let text_lower = text.to_lowercase();

    if text_lower.contains("today") {
        return Some(Utc::now());
    }

    if text_lower.contains("tomorrow") {
        return Some(Utc::now() + Duration::days(1));
    }

    let weekdays = [
        ("monday", Weekday::Mon),
        ("tuesday", Weekday::Tue),
        ("wednesday", Weekday::Wed),
        ("thursday", Weekday::Thu),
        ("friday", Weekday::Fri),
        ("saturday", Weekday::Sat),
        ("sunday", Weekday::Sun),
    ];

    for (name, weekday) in weekdays {
        if text_lower.contains(name) {
            let today = Local::now().date_naive();
            let days_until = (weekday.num_days_from_monday() as i64
                - today.weekday().num_days_from_monday() as i64
                + 7)
                % 7;
            let target_date = today + Duration::days(if days_until == 0 { 7 } else { days_until });
            let dt = target_date.and_hms_opt(9, 0, 0)?;
            return Some(Local.from_local_datetime(&dt).single()?.with_timezone(&Utc));
        }
    }

    let time_re = Regex::new(r"(\d{1,2})\s*(am|pm)").ok()?;
    if let Some(caps) = time_re.captures(&text_lower) {
        let hour: u32 = caps.get(1)?.as_str().parse().ok()?;
        let is_pm = caps.get(2)?.as_str() == "pm";

        let hour = if is_pm && hour != 12 {
            hour + 12
        } else if !is_pm && hour == 12 {
            0
        } else {
            hour
        };

        let today = Local::now().date_naive();
        let time = NaiveTime::from_hms_opt(hour, 0, 0)?;
        let dt = today.and_time(time);
        return Some(Local.from_local_datetime(&dt).single()?.with_timezone(&Utc));
    }

    let at_time_re = Regex::new(r"at\s+(\d{1,2})(?::(\d{2}))?").ok()?;
    if let Some(caps) = at_time_re.captures(&text_lower) {
        let hour: u32 = caps.get(1)?.as_str().parse().ok()?;
        let minute: u32 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let today = Local::now().date_naive();
        let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
        let dt = today.and_time(time);
        return Some(Local.from_local_datetime(&dt).single()?.with_timezone(&Utc));
    }

    None
}

fn parse_chinese_time(text: &str) -> Option<DateTime<Utc>> {
    let now = Local::now();
    let today = now.date_naive();

    let mut date = today;
    let mut hour: u32 = 9;

    if text.contains("今天") {
    } else if text.contains("明天") {
        date = today + Duration::days(1);
    } else if text.contains("后天") {
        date = today + Duration::days(2);
    } else if text.contains("下周") {
        date = today + Duration::days(7);
    }

    let weekday_map = [
        ("周一", Weekday::Mon),
        ("周二", Weekday::Tue),
        ("周三", Weekday::Wed),
        ("周四", Weekday::Thu),
        ("周五", Weekday::Fri),
        ("周六", Weekday::Sat),
        ("周日", Weekday::Sun),
        ("星期一", Weekday::Mon),
        ("星期二", Weekday::Tue),
        ("星期三", Weekday::Wed),
        ("星期四", Weekday::Thu),
        ("星期五", Weekday::Fri),
        ("星期六", Weekday::Sat),
        ("星期日", Weekday::Sun),
    ];

    for (pattern, weekday) in weekday_map {
        if text.contains(pattern) {
            let days_until = (weekday.num_days_from_monday() as i64
                - today.weekday().num_days_from_monday() as i64
                + 7)
                % 7;
            date = today + Duration::days(if days_until == 0 { 7 } else { days_until });
            break;
        }
    }

    if text.contains("早上") || text.contains("上午") {
        hour = 9;
    } else if text.contains("中午") {
        hour = 12;
    } else if text.contains("下午") {
        hour = 14;
    } else if text.contains("晚上") {
        hour = 19;
    }

    let time_re = Regex::new(r"(\d{1,2})[点時时](?:(\d{2})?分?)?").ok()?;
    if let Some(caps) = time_re.captures(text) {
        hour = caps.get(1)?.as_str().parse().ok()?;
    }

    let time = NaiveTime::from_hms_opt(hour, 0, 0)?;
    let dt = date.and_time(time);
    Some(Local.from_local_datetime(&dt).single()?.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_time_parsing() {
        assert!(parse_time_expression("tomorrow").is_some());
        assert!(parse_time_expression("today").is_some());
        assert!(parse_time_expression("at 3pm").is_some());
    }

    #[test]
    fn test_chinese_time_parsing() {
        assert!(parse_time_expression("明天").is_some());
        assert!(parse_time_expression("今天下午").is_some());
        assert!(parse_time_expression("下周一").is_some());
    }

    #[test]
    fn test_parse_at_time_chinese() {
        let result = parse_at_time_expression("买牛奶 @明天早上8点");
        assert!(result.is_some());
        // Verify it's in the future (basic sanity check)
        let dt = result.unwrap();
        assert!(dt > Utc::now() - Duration::days(1));
    }

    #[test]
    fn test_parse_at_time_english() {
        let result = parse_at_time_expression("Buy milk @tomorrow");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_at_time_with_am_pm() {
        let result = parse_at_time_expression("Meeting @3pm");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_at_time_with_punctuation() {
        let result = parse_at_time_expression("Meeting @3pm, downtown");
        assert!(result.is_some());
        // Should stop at comma
    }

    #[test]
    fn test_parse_at_time_invalid() {
        let result = parse_at_time_expression("Email @john about the meeting");
        assert!(result.is_none());
        // "john" is not a valid time expression
    }

    #[test]
    fn test_extract_at_time() {
        assert_eq!(extract_at_time("Task @明天"), Some("明天".to_string()));
        assert_eq!(
            extract_at_time("Meeting @3pm in room"),
            Some("3pm".to_string())
        );
        assert_eq!(extract_at_time("No symbol"), None);
        assert_eq!(
            extract_at_time("Meeting @tomorrow at 5pm"),
            Some("tomorrow".to_string())
        );
    }

    #[test]
    fn test_extract_at_time_stops_at_comma() {
        assert_eq!(
            extract_at_time("Task @9am, room 5"),
            Some("9am".to_string())
        );
    }
}

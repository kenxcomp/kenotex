use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc, Weekday};
use chrono_english::{Dialect, parse_date_string};
use regex::Regex;

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
}

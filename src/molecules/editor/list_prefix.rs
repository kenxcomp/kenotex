/// Pure functions for detecting and manipulating list prefixes in text lines.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPrefix {
    /// Leading whitespace before the prefix.
    pub indent: String,
    /// The continuation string for the next line (e.g. "- [ ] ", "- ", "2. ").
    pub continuation: String,
}

/// Detect list prefix on a line and return the continuation for the next line.
///
/// Supported patterns:
/// - `- [ ] ` / `- [x] ` / `- [X] ` → continuation `- [ ] ` (always unchecked)
/// - `- ` → continuation `- `
/// - `N. ` → continuation `(N+1). `
/// - `N) ` → continuation `(N+1)) `
///
/// Leading whitespace (indent) is preserved.
pub fn detect_list_prefix(line: &str) -> Option<ListPrefix> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = line[..indent_len].to_string();

    // Checkbox: - [ ] , - [x] , - [X]
    if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
    {
        return Some(ListPrefix {
            indent,
            continuation: "- [ ] ".to_string(),
        });
    }

    // Plain checkbox without trailing content (prefix-only handled separately)
    if trimmed == "- [ ]" || trimmed == "- [x]" || trimmed == "- [X]" {
        return Some(ListPrefix {
            indent,
            continuation: "- [ ] ".to_string(),
        });
    }

    // Ordered list: N. or N)
    // Must check before unordered dash to avoid matching "- " inside "1. - "
    if let Some(cont) = try_ordered_prefix(trimmed, '.') {
        return Some(ListPrefix {
            indent,
            continuation: cont,
        });
    }
    if let Some(cont) = try_ordered_prefix(trimmed, ')') {
        return Some(ListPrefix {
            indent,
            continuation: cont,
        });
    }

    // Unordered: - (must come after checkbox check)
    if trimmed.starts_with("- ") {
        return Some(ListPrefix {
            indent,
            continuation: "- ".to_string(),
        });
    }

    // Bare dash (prefix-only case)
    if trimmed == "-" {
        return Some(ListPrefix {
            indent,
            continuation: "- ".to_string(),
        });
    }

    None
}

/// Try to parse an ordered list prefix with the given delimiter ('.' or ')').
/// Returns the continuation string like "2. " or "2) ".
fn try_ordered_prefix(trimmed: &str, delim: char) -> Option<String> {
    let delim_pos = trimmed.find(delim)?;
    if delim_pos == 0 {
        return None;
    }
    let num_str = &trimmed[..delim_pos];
    let num: u64 = num_str.parse().ok()?;
    // After the delimiter, there must be a space (or it's the entire line for prefix-only)
    let after_delim = &trimmed[delim_pos + 1..];
    if after_delim.is_empty() || after_delim.starts_with(' ') {
        Some(format!("{}{} ", num + 1, delim))
    } else {
        None
    }
}

/// Check if a line consists only of a list prefix with no content after it.
///
/// For example, `- [ ] ` or `  - ` or `1. ` are prefix-only lines.
pub fn is_prefix_only(line: &str) -> bool {
    let trimmed = line.trim_start();

    // Checkbox prefix-only
    if trimmed == "- [ ] " || trimmed == "- [x] " || trimmed == "- [X] "
        || trimmed == "- [ ]" || trimmed == "- [x]" || trimmed == "- [X]"
    {
        return true;
    }

    // Unordered dash prefix-only
    if trimmed == "- " || trimmed == "-" {
        return true;
    }

    // Ordered prefix-only: N. or N) with optional trailing space
    let trimmed_end = trimmed.trim_end();
    if let Some(delim_pos) = trimmed_end.rfind('.')
        && delim_pos > 0
        && trimmed_end[..delim_pos].chars().all(|c| c.is_ascii_digit())
    {
        let after = &trimmed[delim_pos + 1..];
        if after.is_empty() || after == " " {
            return true;
        }
    }
    if let Some(delim_pos) = trimmed_end.rfind(')')
        && delim_pos > 0
        && trimmed_end[..delim_pos].chars().all(|c| c.is_ascii_digit())
    {
        let after = &trimmed[delim_pos + 1..];
        if after.is_empty() || after == " " {
            return true;
        }
    }

    false
}

/// Check if a line already has a checkbox prefix (`- [ ] `, `- [x] `, `- [X] `).
pub fn has_checkbox_prefix(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
        || trimmed == "- [ ]"
        || trimmed == "- [x]"
        || trimmed == "- [X]"
}

/// Toggle a checkbox between checked and unchecked.
///
/// - `- [ ] ` → `- [x] ` (check)
/// - `- [x] ` / `- [X] ` → `- [ ] ` (uncheck)
/// - No checkbox → `None`
///
/// Preserves leading indentation.
pub fn toggle_checkbox_prefix(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if trimmed.starts_with("- [ ] ") {
        Some(format!("{}- [x] {}", indent, &trimmed[6..]))
    } else if trimmed == "- [ ]" {
        Some(format!("{}- [x]", indent))
    } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        Some(format!("{}- [ ] {}", indent, &trimmed[6..]))
    } else if trimmed == "- [x]" || trimmed == "- [X]" {
        Some(format!("{}- [ ]", indent))
    } else {
        None
    }
}

/// Prepend `- [ ] ` after indent on a line. Returns `None` if a checkbox already exists.
pub fn insert_checkbox_prefix(line: &str) -> Option<String> {
    if has_checkbox_prefix(line) {
        return None;
    }

    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    Some(format!("{}- [ ] {}", indent, trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_list_prefix ──────────────────────────────────────────

    #[test]
    fn test_detect_checkbox_unchecked() {
        let p = detect_list_prefix("- [ ] buy milk").unwrap();
        assert_eq!(p.indent, "");
        assert_eq!(p.continuation, "- [ ] ");
    }

    #[test]
    fn test_detect_checkbox_checked_x() {
        let p = detect_list_prefix("- [x] done task").unwrap();
        assert_eq!(p.continuation, "- [ ] ");
    }

    #[test]
    fn test_detect_checkbox_checked_upper_x() {
        let p = detect_list_prefix("- [X] done").unwrap();
        assert_eq!(p.continuation, "- [ ] ");
    }

    #[test]
    fn test_detect_unordered_dash() {
        let p = detect_list_prefix("- some item").unwrap();
        assert_eq!(p.continuation, "- ");
    }

    #[test]
    fn test_detect_ordered_dot() {
        let p = detect_list_prefix("1. first item").unwrap();
        assert_eq!(p.continuation, "2. ");
    }

    #[test]
    fn test_detect_ordered_paren() {
        let p = detect_list_prefix("3) third item").unwrap();
        assert_eq!(p.continuation, "4) ");
    }

    #[test]
    fn test_detect_indented_checkbox() {
        let p = detect_list_prefix("    - [ ] indented").unwrap();
        assert_eq!(p.indent, "    ");
        assert_eq!(p.continuation, "- [ ] ");
    }

    #[test]
    fn test_detect_no_prefix() {
        assert!(detect_list_prefix("just some text").is_none());
        assert!(detect_list_prefix("").is_none());
    }

    // ── is_prefix_only ──────────────────────────────────────────────

    #[test]
    fn test_prefix_only_checkbox() {
        assert!(is_prefix_only("- [ ] "));
        assert!(is_prefix_only("- [x] "));
        assert!(is_prefix_only("  - [ ] "));
    }

    #[test]
    fn test_prefix_only_dash() {
        assert!(is_prefix_only("- "));
        assert!(is_prefix_only("  - "));
    }

    #[test]
    fn test_prefix_only_ordered() {
        assert!(is_prefix_only("1. "));
        assert!(is_prefix_only("2) "));
    }

    #[test]
    fn test_not_prefix_only() {
        assert!(!is_prefix_only("- [ ] task"));
        assert!(!is_prefix_only("- item"));
        assert!(!is_prefix_only("1. first"));
    }

    // ── has_checkbox_prefix ─────────────────────────────────────────

    #[test]
    fn test_has_checkbox_true() {
        assert!(has_checkbox_prefix("- [ ] something"));
        assert!(has_checkbox_prefix("- [x] done"));
        assert!(has_checkbox_prefix("  - [X] done"));
    }

    #[test]
    fn test_has_checkbox_false() {
        assert!(!has_checkbox_prefix("- item"));
        assert!(!has_checkbox_prefix("1. first"));
        assert!(!has_checkbox_prefix("plain text"));
    }

    // ── insert_checkbox_prefix ──────────────────────────────────────

    #[test]
    fn test_insert_checkbox_on_plain() {
        assert_eq!(
            insert_checkbox_prefix("buy milk"),
            Some("- [ ] buy milk".to_string())
        );
    }

    #[test]
    fn test_insert_checkbox_preserves_indent() {
        assert_eq!(
            insert_checkbox_prefix("    indented"),
            Some("    - [ ] indented".to_string())
        );
    }

    #[test]
    fn test_insert_checkbox_already_exists() {
        assert_eq!(insert_checkbox_prefix("- [ ] already"), None);
        assert_eq!(insert_checkbox_prefix("- [x] done"), None);
    }

    #[test]
    fn test_insert_checkbox_on_empty() {
        assert_eq!(
            insert_checkbox_prefix(""),
            Some("- [ ] ".to_string())
        );
    }

    // ── toggle_checkbox_prefix ────────────────────────────────────────

    #[test]
    fn test_toggle_checkbox_check() {
        assert_eq!(
            toggle_checkbox_prefix("- [ ] buy milk"),
            Some("- [x] buy milk".to_string())
        );
    }

    #[test]
    fn test_toggle_checkbox_uncheck_lower() {
        assert_eq!(
            toggle_checkbox_prefix("- [x] done task"),
            Some("- [ ] done task".to_string())
        );
    }

    #[test]
    fn test_toggle_checkbox_uncheck_upper() {
        assert_eq!(
            toggle_checkbox_prefix("- [X] done"),
            Some("- [ ] done".to_string())
        );
    }

    #[test]
    fn test_toggle_checkbox_preserves_indent() {
        assert_eq!(
            toggle_checkbox_prefix("    - [ ] indented"),
            Some("    - [x] indented".to_string())
        );
        assert_eq!(
            toggle_checkbox_prefix("    - [x] indented"),
            Some("    - [ ] indented".to_string())
        );
    }

    #[test]
    fn test_toggle_checkbox_bare() {
        assert_eq!(
            toggle_checkbox_prefix("- [ ]"),
            Some("- [x]".to_string())
        );
        assert_eq!(
            toggle_checkbox_prefix("- [x]"),
            Some("- [ ]".to_string())
        );
    }

    #[test]
    fn test_toggle_checkbox_no_checkbox() {
        assert_eq!(toggle_checkbox_prefix("- plain item"), None);
        assert_eq!(toggle_checkbox_prefix("just text"), None);
        assert_eq!(toggle_checkbox_prefix(""), None);
    }
}

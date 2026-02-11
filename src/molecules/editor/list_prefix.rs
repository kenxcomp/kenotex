//! Pure functions for detecting and manipulating list prefixes in text lines.

use unicode_width::UnicodeWidthStr;

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

    // Unordered: * (must come after ordered check to avoid matching "1. * ")
    if trimmed.starts_with("* ") {
        return Some(ListPrefix {
            indent,
            continuation: "* ".to_string(),
        });
    }

    // Bare asterisk (prefix-only case)
    if trimmed == "*" {
        return Some(ListPrefix {
            indent,
            continuation: "* ".to_string(),
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
    if trimmed == "- [ ] "
        || trimmed == "- [x] "
        || trimmed == "- [X] "
        || trimmed == "- [ ]"
        || trimmed == "- [x]"
        || trimmed == "- [X]"
    {
        return true;
    }

    // Unordered dash prefix-only
    if trimmed == "- " || trimmed == "-" {
        return true;
    }

    // Unordered asterisk prefix-only
    if trimmed == "* " || trimmed == "*" {
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

/// Compute the display width of the current line's prefix for hanging indent.
///
/// Returns `indent_display_width + prefix_display_width`, or `0` if no prefix.
/// Unlike `detect_list_prefix()` which returns the *next* line's continuation,
/// this measures the *current* line's actual prefix width.
///
/// Supported prefixes: `- [ ] `, `- [x] `, `- [X] `, `- `, `* `, `N. `, `N) `
pub fn hanging_indent_width(line: &str) -> usize {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];
    let indent_dw = indent.width();

    // Checkbox: - [ ] , - [x] , - [X]  → 6 display columns
    if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
    {
        return indent_dw + 6;
    }

    // Unordered: "- " → 2 display columns
    if trimmed.starts_with("- ") {
        return indent_dw + 2;
    }

    // Unordered: "* " → 2 display columns
    if trimmed.starts_with("* ") {
        return indent_dw + 2;
    }

    // Ordered: N. or N) → digit_count + 2 display columns
    let mut digit_count = 0;
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            digit_count += 1;
        } else {
            break;
        }
    }
    if digit_count > 0 {
        let rest = &trimmed[digit_count..];
        if rest.starts_with(". ") || rest.starts_with(") ") {
            return indent_dw + digit_count + 2;
        }
    }

    0
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

    if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
        Some(format!("{}- [x] {}", indent, rest))
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
        assert_eq!(insert_checkbox_prefix(""), Some("- [ ] ".to_string()));
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
        assert_eq!(toggle_checkbox_prefix("- [ ]"), Some("- [x]".to_string()));
        assert_eq!(toggle_checkbox_prefix("- [x]"), Some("- [ ]".to_string()));
    }

    #[test]
    fn test_toggle_checkbox_no_checkbox() {
        assert_eq!(toggle_checkbox_prefix("- plain item"), None);
        assert_eq!(toggle_checkbox_prefix("just text"), None);
        assert_eq!(toggle_checkbox_prefix(""), None);
    }

    // ── detect_list_prefix: asterisk ──────────────────────────────────

    #[test]
    fn test_detect_asterisk() {
        let p = detect_list_prefix("* some item").unwrap();
        assert_eq!(p.indent, "");
        assert_eq!(p.continuation, "* ");
    }

    #[test]
    fn test_detect_indented_asterisk() {
        let p = detect_list_prefix("    * nested item").unwrap();
        assert_eq!(p.indent, "    ");
        assert_eq!(p.continuation, "* ");
    }

    #[test]
    fn test_detect_bare_asterisk() {
        let p = detect_list_prefix("*").unwrap();
        assert_eq!(p.continuation, "* ");
    }

    // ── is_prefix_only: asterisk ──────────────────────────────────────

    #[test]
    fn test_prefix_only_asterisk() {
        assert!(is_prefix_only("* "));
        assert!(is_prefix_only("  * "));
        assert!(is_prefix_only("*"));
    }

    #[test]
    fn test_not_prefix_only_asterisk() {
        assert!(!is_prefix_only("* item"));
    }

    // ── hanging_indent_width ──────────────────────────────────────────

    #[test]
    fn test_hanging_indent_checkbox() {
        assert_eq!(hanging_indent_width("- [ ] buy milk"), 6);
        assert_eq!(hanging_indent_width("- [x] done"), 6);
        assert_eq!(hanging_indent_width("- [X] done"), 6);
    }

    #[test]
    fn test_hanging_indent_checkbox_indented() {
        assert_eq!(hanging_indent_width("  - [ ] indented"), 8);
        assert_eq!(hanging_indent_width("    - [x] deeply"), 10);
    }

    #[test]
    fn test_hanging_indent_dash() {
        assert_eq!(hanging_indent_width("- item"), 2);
        assert_eq!(hanging_indent_width("  - item"), 4);
    }

    #[test]
    fn test_hanging_indent_asterisk() {
        assert_eq!(hanging_indent_width("* item"), 2);
        assert_eq!(hanging_indent_width("  * item"), 4);
    }

    #[test]
    fn test_hanging_indent_ordered_dot() {
        assert_eq!(hanging_indent_width("1. first"), 3);
        assert_eq!(hanging_indent_width("10. tenth"), 4);
        assert_eq!(hanging_indent_width("  1. indented"), 5);
    }

    #[test]
    fn test_hanging_indent_ordered_paren() {
        assert_eq!(hanging_indent_width("1) first"), 3);
        assert_eq!(hanging_indent_width("99) big"), 4);
    }

    #[test]
    fn test_hanging_indent_no_prefix() {
        assert_eq!(hanging_indent_width("just text"), 0);
        assert_eq!(hanging_indent_width(""), 0);
        assert_eq!(hanging_indent_width("   indented text"), 0);
    }

    #[test]
    fn test_hanging_indent_cjk_content() {
        // CJK content after prefix should not affect indent width
        assert_eq!(hanging_indent_width("- 你好世界"), 2);
        assert_eq!(hanging_indent_width("  - [ ] 买牛奶"), 8);
    }
}

//! Pure functions for detecting and toggling HTML comment markers (`<!-- -->`).

/// Check if a line (after trimming) is an HTML comment.
pub fn is_commented(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<!--") && trimmed.ends_with("-->")
}

/// Wrap a line in `<!-- -->`, preserving leading indentation.
pub fn comment_line(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return line.to_string();
    }
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];
    format!("{}<!-- {} -->", indent, trimmed)
}

/// Strip `<!-- -->` from a commented line, preserving indentation.
/// Returns `None` if the line is not commented.
pub fn uncomment_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
        return None;
    }

    let inner = &trimmed[4..trimmed.len() - 3];
    // Strip one leading and one trailing space if present
    let inner = inner.strip_prefix(' ').unwrap_or(inner);
    let inner = inner.strip_suffix(' ').unwrap_or(inner);

    Some(format!("{}{}", indent, inner))
}

/// Toggle comment on a single line: comment if uncommented, uncomment if commented.
pub fn toggle_comment_line(line: &str) -> String {
    if is_commented(line) {
        uncomment_line(line).unwrap_or_else(|| line.to_string())
    } else {
        comment_line(line)
    }
}

/// Determine whether a set of lines should be commented (true) or uncommented (false).
/// Returns true if any non-empty line is uncommented.
pub fn should_comment(lines: &[&str]) -> bool {
    lines
        .iter()
        .any(|l| !l.trim().is_empty() && !is_commented(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_commented ──────────────────────────────────────────────

    #[test]
    fn test_is_commented_basic() {
        assert!(is_commented("<!-- hello -->"));
        assert!(is_commented("  <!-- hello -->"));
    }

    #[test]
    fn test_is_commented_no_spaces() {
        assert!(is_commented("<!--hello-->"));
    }

    #[test]
    fn test_is_commented_false() {
        assert!(!is_commented("hello"));
        assert!(!is_commented("<!-- half"));
        assert!(!is_commented("half -->"));
        assert!(!is_commented(""));
    }

    // ── comment_line ──────────────────────────────────────────────

    #[test]
    fn test_comment_line_basic() {
        assert_eq!(comment_line("hello"), "<!-- hello -->");
    }

    #[test]
    fn test_comment_line_preserves_indent() {
        assert_eq!(comment_line("    hello"), "    <!-- hello -->");
    }

    #[test]
    fn test_comment_line_empty() {
        assert_eq!(comment_line(""), "");
        assert_eq!(comment_line("   "), "   ");
    }

    // ── uncomment_line ────────────────────────────────────────────

    #[test]
    fn test_uncomment_line_basic() {
        assert_eq!(uncomment_line("<!-- hello -->"), Some("hello".to_string()));
    }

    #[test]
    fn test_uncomment_line_preserves_indent() {
        assert_eq!(
            uncomment_line("    <!-- hello -->"),
            Some("    hello".to_string())
        );
    }

    #[test]
    fn test_uncomment_line_no_spaces() {
        assert_eq!(uncomment_line("<!--hello-->"), Some("hello".to_string()));
    }

    #[test]
    fn test_uncomment_line_not_commented() {
        assert_eq!(uncomment_line("hello"), None);
    }

    // ── toggle_comment_line ───────────────────────────────────────

    #[test]
    fn test_toggle_comment_comments() {
        assert_eq!(toggle_comment_line("hello"), "<!-- hello -->");
    }

    #[test]
    fn test_toggle_comment_uncomments() {
        assert_eq!(toggle_comment_line("<!-- hello -->"), "hello");
    }

    #[test]
    fn test_toggle_comment_empty() {
        assert_eq!(toggle_comment_line(""), "");
    }

    #[test]
    fn test_toggle_comment_indented_roundtrip() {
        let original = "    some text";
        let commented = toggle_comment_line(original);
        assert_eq!(commented, "    <!-- some text -->");
        let uncommented = toggle_comment_line(&commented);
        assert_eq!(uncommented, original);
    }

    // ── should_comment ────────────────────────────────────────────

    #[test]
    fn test_should_comment_all_uncommented() {
        assert!(should_comment(&["hello", "world"]));
    }

    #[test]
    fn test_should_comment_all_commented() {
        assert!(!should_comment(&["<!-- hello -->", "<!-- world -->"]));
    }

    #[test]
    fn test_should_comment_mixed() {
        assert!(should_comment(&["<!-- hello -->", "world"]));
    }

    #[test]
    fn test_should_comment_with_empty_lines() {
        assert!(!should_comment(&["<!-- hello -->", "", "<!-- world -->"]));
    }

    #[test]
    fn test_should_comment_all_empty() {
        assert!(!should_comment(&["", "  "]));
    }
}

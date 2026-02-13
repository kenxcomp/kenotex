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

/// Check whether all non-whitespace content is enclosed in HTML comments.
/// Handles both single-line `<!-- ... -->` and multi-line `<!-- ... \n ... -->`.
/// Returns false for empty/whitespace-only text.
pub fn is_all_commented(text: &str) -> bool {
    let mut inside_comment = false;
    let mut found_comment = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if inside_comment {
            if trimmed.ends_with("-->") {
                inside_comment = false;
            }
            // Inside comment body — continue
        } else {
            if trimmed.is_empty() {
                continue; // whitespace-only lines are OK outside comments
            }
            if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
                // Single-line comment
                found_comment = true;
            } else if trimmed.starts_with("<!--") {
                // Opens a multi-line comment
                inside_comment = true;
                found_comment = true;
            } else {
                // Uncommented content found
                return false;
            }
        }
    }

    found_comment && !inside_comment
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

    // ── is_all_commented ────────────────────────────────────────────

    #[test]
    fn test_is_all_commented_single_line() {
        assert!(is_all_commented("<!-- hello -->"));
    }

    #[test]
    fn test_is_all_commented_multi_line() {
        assert!(is_all_commented("<!-- :::td\n- Buy milk\n::: -->"));
    }

    #[test]
    fn test_is_all_commented_multiple_blocks() {
        assert!(is_all_commented(
            "<!-- :::td\n- Buy milk\n::: -->\n\n<!-- :::cal\nMeeting\n::: -->"
        ));
    }

    #[test]
    fn test_is_all_commented_whitespace_between_blocks() {
        assert!(is_all_commented("<!-- block1 -->\n\n  \n\n<!-- block2 -->"));
    }

    #[test]
    fn test_is_all_commented_mixed_content() {
        assert!(!is_all_commented(
            "<!-- :::td\n- Buy milk\n::: -->\n\nSome uncommented text"
        ));
    }

    #[test]
    fn test_is_all_commented_empty_text() {
        assert!(!is_all_commented(""));
    }

    #[test]
    fn test_is_all_commented_whitespace_only() {
        assert!(!is_all_commented("   \n  \n   "));
    }

    #[test]
    fn test_is_all_commented_unclosed_comment() {
        assert!(!is_all_commented("<!-- unclosed comment"));
    }

    #[test]
    fn test_is_all_commented_indented_comments() {
        assert!(is_all_commented("  <!-- hello -->"));
    }

    #[test]
    fn test_is_all_commented_cjk_content() {
        assert!(is_all_commented(
            "<!-- :::td\n- 买牛奶 @明天早上8点\n::: -->"
        ));
    }

    #[test]
    fn test_is_all_commented_realistic_post_processing() {
        let text = "<!-- :::td\n- Buy milk @tomorrow\n- Walk dog @9pm\n::: -->\n\n<!-- :::cal\nTeam meeting @下周一上午9点\nRoom 301\n::: -->";
        assert!(is_all_commented(text));
    }

    #[test]
    fn test_is_all_commented_partial_processing() {
        let text = "<!-- :::td\n- Buy milk @tomorrow\n::: -->\n\n:::cal\nTeam meeting\n:::";
        assert!(!is_all_commented(text));
    }

    #[test]
    fn test_is_all_commented_only_whitespace_outside() {
        assert!(is_all_commented("\n\n<!-- hello -->\n\n"));
    }
}

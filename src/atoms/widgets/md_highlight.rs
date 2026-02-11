/// Markdown inline syntax tokenizer for editor highlighting.
/// Pure tokenizer function with no dependencies on ratatui or theme types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdTokenKind {
    Plain,
    Bold,
    Italic,
    BoldItalic,
    Strikethrough,
    InlineCode,
    Delimiter,
    OrderedListPrefix,
    UnorderedListPrefix,
    TimeExpression, // NEW: for @time highlighting
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdToken {
    pub text: String,
    pub kind: MdTokenKind,
}

/// Tokenizes a line of text into markdown inline elements.
/// Single left-to-right pass with delimiter priority: ` > ~~ > *** > ** > *
pub fn tokenize_inline(line: &str) -> Vec<MdToken> {
    let mut tokens = Vec::new();

    // Check for list prefix at line start
    let remaining = if let Some(prefix_end) = check_list_prefix(line) {
        let (prefix, rest) = line.split_at(prefix_end);
        let trimmed_prefix = prefix.trim_start();
        let kind = if trimmed_prefix.starts_with('-') || trimmed_prefix.starts_with('*') {
            MdTokenKind::UnorderedListPrefix
        } else {
            MdTokenKind::OrderedListPrefix
        };
        tokens.push(MdToken {
            text: prefix.to_string(),
            kind,
        });
        rest
    } else {
        line
    };

    // Parse inline formatting
    let mut i = 0;
    let chars: Vec<char> = remaining.chars().collect();
    let mut plain_buffer = String::new();

    while i < chars.len() {
        let mut handled = false;

        // Try @time expression FIRST (highest priority after list prefix)
        if chars[i] == '@' {
            if let Some((time_text, end)) = scan_time_expression(&chars, i) {
                if !plain_buffer.is_empty() {
                    tokens.push(MdToken {
                        text: plain_buffer.clone(),
                        kind: MdTokenKind::Plain,
                    });
                    plain_buffer.clear();
                }
                tokens.push(MdToken {
                    text: format!("@{}", time_text),
                    kind: MdTokenKind::TimeExpression,
                });
                i = end;
                handled = true;
            }
        }

        // Try bold italic first (longest * sequence)
        if !handled
            && i + 2 < chars.len()
            && chars[i] == '*'
            && chars[i + 1] == '*'
            && chars[i + 2] == '*'
        {
            match scan_delimited(&chars, i, "***") {
                Some((content, end)) => {
                    if !plain_buffer.is_empty() {
                        tokens.push(MdToken {
                            text: plain_buffer.clone(),
                            kind: MdTokenKind::Plain,
                        });
                        plain_buffer.clear();
                    }
                    tokens.push(MdToken {
                        text: "***".to_string(),
                        kind: MdTokenKind::Delimiter,
                    });
                    tokens.push(MdToken {
                        text: content,
                        kind: MdTokenKind::BoldItalic,
                    });
                    tokens.push(MdToken {
                        text: "***".to_string(),
                        kind: MdTokenKind::Delimiter,
                    });
                    i = end;
                    handled = true;
                }
                None => {
                    // No closing ***, skip checking shorter ** and * at this position
                    plain_buffer.push(chars[i]);
                    i += 1;
                    handled = true;
                }
            }
        }

        // Try bold
        if !handled && i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            match scan_delimited(&chars, i, "**") {
                Some((content, end)) => {
                    if !plain_buffer.is_empty() {
                        tokens.push(MdToken {
                            text: plain_buffer.clone(),
                            kind: MdTokenKind::Plain,
                        });
                        plain_buffer.clear();
                    }
                    tokens.push(MdToken {
                        text: "**".to_string(),
                        kind: MdTokenKind::Delimiter,
                    });
                    tokens.push(MdToken {
                        text: content,
                        kind: MdTokenKind::Bold,
                    });
                    tokens.push(MdToken {
                        text: "**".to_string(),
                        kind: MdTokenKind::Delimiter,
                    });
                    i = end;
                    handled = true;
                }
                None => {
                    // No closing **, skip checking shorter * at this position
                    plain_buffer.push(chars[i]);
                    i += 1;
                    handled = true;
                }
            }
        }

        // Try italic (single *)
        if !handled
            && chars[i] == '*'
            && let Some((content, end)) = scan_delimited(&chars, i, "*")
        {
            if !plain_buffer.is_empty() {
                tokens.push(MdToken {
                    text: plain_buffer.clone(),
                    kind: MdTokenKind::Plain,
                });
                plain_buffer.clear();
            }
            tokens.push(MdToken {
                text: "*".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            tokens.push(MdToken {
                text: content,
                kind: MdTokenKind::Italic,
            });
            tokens.push(MdToken {
                text: "*".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            i = end;
            handled = true;
        }

        // Try inline code
        if !handled
            && chars[i] == '`'
            && let Some((content, end)) = scan_delimited(&chars, i, "`")
        {
            if !plain_buffer.is_empty() {
                tokens.push(MdToken {
                    text: plain_buffer.clone(),
                    kind: MdTokenKind::Plain,
                });
                plain_buffer.clear();
            }
            tokens.push(MdToken {
                text: "`".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            tokens.push(MdToken {
                text: content,
                kind: MdTokenKind::InlineCode,
            });
            tokens.push(MdToken {
                text: "`".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            i = end;
            handled = true;
        }

        // Try strikethrough
        if !handled
            && i + 1 < chars.len()
            && chars[i] == '~'
            && chars[i + 1] == '~'
            && let Some((content, end)) = scan_delimited(&chars, i, "~~")
        {
            if !plain_buffer.is_empty() {
                tokens.push(MdToken {
                    text: plain_buffer.clone(),
                    kind: MdTokenKind::Plain,
                });
                plain_buffer.clear();
            }
            tokens.push(MdToken {
                text: "~~".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            tokens.push(MdToken {
                text: content,
                kind: MdTokenKind::Strikethrough,
            });
            tokens.push(MdToken {
                text: "~~".to_string(),
                kind: MdTokenKind::Delimiter,
            });
            i = end;
            handled = true;
        }

        // Plain character (if no delimiter matched)
        if !handled {
            plain_buffer.push(chars[i]);
            i += 1;
        }
    }

    if !plain_buffer.is_empty() {
        tokens.push(MdToken {
            text: plain_buffer,
            kind: MdTokenKind::Plain,
        });
    }

    tokens
}

/// Checks if line starts with ordered (N. or N)) or unordered (-) list prefix.
/// Returns the byte index after the prefix if found.
fn check_list_prefix(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();

    // Unordered: "- "
    if trimmed.starts_with("- ") {
        return Some(indent_len + 2);
    }

    // Unordered: "* " (must check before bold/italic * handling)
    if trimmed.starts_with("* ") {
        return Some(indent_len + 2);
    }

    // Ordered: "N. " or "N) "
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
        if rest.starts_with(". ") {
            return Some(indent_len + digit_count + 2);
        }
        if rest.starts_with(") ") {
            return Some(indent_len + digit_count + 2);
        }
    }

    None
}

/// Scans for closing delimiter and returns (content, next_index).
/// Returns None if no closing delimiter found.
fn scan_delimited(chars: &[char], start: usize, delim: &str) -> Option<(String, usize)> {
    let delim_chars: Vec<char> = delim.chars().collect();
    let delim_len = delim_chars.len();
    let mut i = start + delim_len;
    let mut content = String::new();

    while i < chars.len() {
        // Check for closing delimiter
        if i + delim_len <= chars.len() {
            let matches = (0..delim_len).all(|j| chars[i + j] == delim_chars[j]);
            if matches {
                return Some((content, i + delim_len));
            }
        }
        content.push(chars[i]);
        i += 1;
    }

    None
}

/// Scans for @-prefixed time expression.
/// Returns (time_text, next_index) if valid time found.
fn scan_time_expression(chars: &[char], start: usize) -> Option<(String, usize)> {
    if start >= chars.len() || chars[start] != '@' {
        return None;
    }

    let mut i = start + 1; // Skip @
    let mut time_text = String::new();

    // Collect until whitespace, punctuation (except colon for time), or end
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            break;
        }
        // Stop at ASCII punctuation except colon
        if ch.is_ascii_punctuation() && ch != ':' {
            break;
        }
        time_text.push(ch);
        i += 1;
    }

    if time_text.is_empty() {
        return None;
    }

    // Validate: must look like a time expression
    // Simple heuristic: contains digits OR known keywords
    let has_digit = time_text.chars().any(|c| c.is_ascii_digit());
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
    let has_keyword = keywords.iter().any(|kw| time_text.contains(kw));

    if has_digit || has_keyword {
        Some((time_text, i))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let tokens = tokenize_inline("Hello world");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, MdTokenKind::Plain);
        assert_eq!(tokens[0].text, "Hello world");
    }

    #[test]
    fn test_bold() {
        let tokens = tokenize_inline("This is **bold** text");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "This is ");
        assert_eq!(tokens[1].kind, MdTokenKind::Delimiter);
        assert_eq!(tokens[2].kind, MdTokenKind::Bold);
        assert_eq!(tokens[2].text, "bold");
        assert_eq!(tokens[3].kind, MdTokenKind::Delimiter);
        assert_eq!(tokens[4].text, " text");
    }

    #[test]
    fn test_italic() {
        let tokens = tokenize_inline("This is *italic* text");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].kind, MdTokenKind::Italic);
        assert_eq!(tokens[2].text, "italic");
    }

    #[test]
    fn test_bold_italic() {
        let tokens = tokenize_inline("This is ***bold italic*** text");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].kind, MdTokenKind::BoldItalic);
        assert_eq!(tokens[2].text, "bold italic");
    }

    #[test]
    fn test_strikethrough() {
        let tokens = tokenize_inline("This is ~~strikethrough~~ text");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].kind, MdTokenKind::Strikethrough);
        assert_eq!(tokens[2].text, "strikethrough");
    }

    #[test]
    fn test_inline_code() {
        let tokens = tokenize_inline("Use `code` here");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].kind, MdTokenKind::InlineCode);
        assert_eq!(tokens[2].text, "code");
    }

    #[test]
    fn test_code_suppresses_formatting() {
        let tokens = tokenize_inline("`**not bold**`");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, MdTokenKind::Delimiter);
        assert_eq!(tokens[1].kind, MdTokenKind::InlineCode);
        assert_eq!(tokens[1].text, "**not bold**");
        assert_eq!(tokens[2].kind, MdTokenKind::Delimiter);
    }

    #[test]
    fn test_unmatched_delimiters() {
        let tokens = tokenize_inline("This is **unmatched text");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, MdTokenKind::Plain);
        assert_eq!(tokens[0].text, "This is **unmatched text");
    }

    #[test]
    fn test_unordered_list() {
        let tokens = tokenize_inline("- List item");
        assert_eq!(tokens[0].kind, MdTokenKind::UnorderedListPrefix);
        assert_eq!(tokens[0].text, "- ");
        assert_eq!(tokens[1].text, "List item");
    }

    #[test]
    fn test_ordered_list_dot() {
        let tokens = tokenize_inline("1. List item");
        assert_eq!(tokens[0].kind, MdTokenKind::OrderedListPrefix);
        assert_eq!(tokens[0].text, "1. ");
    }

    #[test]
    fn test_ordered_list_paren() {
        let tokens = tokenize_inline("42) List item");
        assert_eq!(tokens[0].kind, MdTokenKind::OrderedListPrefix);
        assert_eq!(tokens[0].text, "42) ");
    }

    #[test]
    fn test_list_with_formatting() {
        let tokens = tokenize_inline("- **Bold** item");
        assert_eq!(tokens[0].kind, MdTokenKind::UnorderedListPrefix);
        assert_eq!(tokens[2].kind, MdTokenKind::Bold);
        assert_eq!(tokens[2].text, "Bold");
    }

    #[test]
    fn test_asterisk_list_prefix() {
        let tokens = tokenize_inline("* List item");
        assert_eq!(tokens[0].kind, MdTokenKind::UnorderedListPrefix);
        assert_eq!(tokens[0].text, "* ");
        assert_eq!(tokens[1].text, "List item");
    }

    #[test]
    fn test_indented_asterisk_list_prefix() {
        let tokens = tokenize_inline("  * Nested item");
        assert_eq!(tokens[0].kind, MdTokenKind::UnorderedListPrefix);
        assert_eq!(tokens[0].text, "  * ");
    }

    #[test]
    fn test_multiple_formatting() {
        let tokens = tokenize_inline("**bold** and *italic* and `code`");
        assert!(tokens.iter().any(|t| t.kind == MdTokenKind::Bold));
        assert!(tokens.iter().any(|t| t.kind == MdTokenKind::Italic));
        assert!(tokens.iter().any(|t| t.kind == MdTokenKind::InlineCode));
    }

    #[test]
    fn test_tokenize_at_time_chinese() {
        let tokens = tokenize_inline("买牛奶 @明天早上8点");

        assert!(tokens.iter().any(|t| t.kind == MdTokenKind::TimeExpression));
        let time_token = tokens
            .iter()
            .find(|t| t.kind == MdTokenKind::TimeExpression)
            .unwrap();
        assert_eq!(time_token.text, "@明天早上8点");
    }

    #[test]
    fn test_tokenize_at_time_english() {
        let tokens = tokenize_inline("Buy milk @tomorrow");

        let time_token = tokens
            .iter()
            .find(|t| t.kind == MdTokenKind::TimeExpression)
            .unwrap();
        assert_eq!(time_token.text, "@tomorrow");
    }

    #[test]
    fn test_tokenize_at_time_boundary() {
        let tokens = tokenize_inline("Meeting @3pm, downtown");

        let time_token = tokens
            .iter()
            .find(|t| t.kind == MdTokenKind::TimeExpression)
            .unwrap();
        assert_eq!(time_token.text, "@3pm"); // Should NOT include comma
    }

    #[test]
    fn test_tokenize_at_username_not_highlighted() {
        let tokens = tokenize_inline("Email @john about meeting");

        // "john" doesn't match time pattern, should be Plain text
        assert!(!tokens.iter().any(|t| t.kind == MdTokenKind::TimeExpression));
    }

    #[test]
    fn test_tokenize_multiple_at_times() {
        let tokens = tokenize_inline("Task @明天 or @下周一");

        let time_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == MdTokenKind::TimeExpression)
            .collect();
        assert_eq!(time_tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_at_time_with_colon() {
        let tokens = tokenize_inline("Meeting @9:30am tomorrow");

        let time_token = tokens
            .iter()
            .find(|t| t.kind == MdTokenKind::TimeExpression)
            .unwrap();
        assert_eq!(time_token.text, "@9:30am");
    }
}

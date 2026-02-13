/// Auto-pair insertion logic for Insert mode.
///
/// Pure detection functions that determine what auto-pair action to take.
/// No buffer mutation happens here.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairAction {
    /// Insert closing char after cursor (e.g., `(` inserts `)`)
    InsertPair(char),
    /// Extend existing pair (e.g., `*|*` + `*` → `**|**`)
    Absorb(char),
    /// Convert backtick pair to code block
    CodeBlock,
    /// Move cursor past existing closing char
    Skip,
    /// Normal insert, no auto-pair
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackspaceAction {
    /// Delete both opening and closing chars
    DeletePair,
    /// Normal backspace
    None,
}

/// Determine the auto-pair action for a character insertion.
///
/// Parameters are graphemes around the cursor:
/// - `before_1`: grapheme immediately before cursor
/// - `after_1`: grapheme immediately after cursor
/// - `before_2`: grapheme two positions before cursor
/// - `after_2`: grapheme two positions after cursor (i.e., offset +1)
pub fn on_char_insert(
    c: char,
    before_1: Option<&str>,
    after_1: Option<&str>,
    before_2: Option<&str>,
    after_2: Option<&str>,
) -> PairAction {
    match c {
        // Brackets: always insert pair
        '(' => PairAction::InsertPair(')'),
        '[' => PairAction::InsertPair(']'),
        '{' => PairAction::InsertPair('}'),

        // Closing brackets: skip if already there
        ')' | ']' | '}' => {
            if after_1 == Some(&c.to_string()) {
                PairAction::Skip
            } else {
                PairAction::None
            }
        }

        // Quotes: skip or insert pair
        '\'' | '"' => {
            let cs = c.to_string();
            if after_1 == Some(cs.as_str()) {
                PairAction::Skip
            } else {
                PairAction::InsertPair(c)
            }
        }

        // Asterisk: markdown bold/italic
        '*' => handle_markdown_pair('*', before_1, after_1, before_2, after_2),

        // Tilde: markdown strikethrough
        '~' => handle_markdown_pair('~', before_1, after_1, before_2, after_2),

        // Backtick: inline code / code block
        '`' => {
            // Check for code block: `` `|` `` → inserting third backtick
            if before_2 == Some("`")
                && before_1 == Some("`")
                && after_1 == Some("`")
                && after_2 == Some("`")
            {
                PairAction::CodeBlock
            } else if before_1 == Some("`") && after_1 == Some("`") {
                // Inside a backtick pair: absorb to extend
                PairAction::Absorb('`')
            } else if after_1 == Some("`") {
                PairAction::Skip
            } else {
                PairAction::InsertPair('`')
            }
        }

        _ => PairAction::None,
    }
}

/// Shared logic for `*` and `~` markdown pairs.
fn handle_markdown_pair(
    ch: char,
    before_1: Option<&str>,
    after_1: Option<&str>,
    before_2: Option<&str>,
    after_2: Option<&str>,
) -> PairAction {
    let cs = &ch.to_string();
    let b1 = before_1 == Some(cs.as_str());
    let a1 = after_1 == Some(cs.as_str());
    let b2 = before_2 == Some(cs.as_str());
    let a2 = after_2 == Some(cs.as_str());

    if b1 && a1 && b2 && a2 {
        // At `**|**`, just skip
        PairAction::Skip
    } else if b1 && a1 && !(b2 && a2) {
        // At `*|*` (but not `**|**`), absorb to extend
        PairAction::Absorb(ch)
    } else if a1 {
        PairAction::Skip
    } else {
        PairAction::InsertPair(ch)
    }
}

/// Determine the backspace action when cursor is between two characters.
///
/// If `before_1` and `after_1` form a matching pair, return `DeletePair`.
pub fn on_backspace(before_1: Option<&str>, after_1: Option<&str>) -> BackspaceAction {
    match (before_1, after_1) {
        (Some("("), Some(")"))
        | (Some("["), Some("]"))
        | (Some("{"), Some("}"))
        | (Some("'"), Some("'"))
        | (Some("\""), Some("\""))
        | (Some("`"), Some("`"))
        | (Some("*"), Some("*"))
        | (Some("~"), Some("~")) => BackspaceAction::DeletePair,
        _ => BackspaceAction::None,
    }
}

/// Check if a line is an opening `:::` tag that should auto-close.
pub fn is_opening_tag(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == ":::td" || trimmed == ":::cal" || trimmed == ":::note"
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bracket pairs (opening) ────────────────────────────────────

    #[test]
    fn test_open_paren_always_inserts_pair() {
        assert_eq!(
            on_char_insert('(', None, None, None, None),
            PairAction::InsertPair(')')
        );
    }

    #[test]
    fn test_open_bracket_always_inserts_pair() {
        assert_eq!(
            on_char_insert('[', None, None, None, None),
            PairAction::InsertPair(']')
        );
    }

    #[test]
    fn test_open_brace_always_inserts_pair() {
        assert_eq!(
            on_char_insert('{', None, None, None, None),
            PairAction::InsertPair('}')
        );
    }

    #[test]
    fn test_open_paren_with_surrounding_text() {
        assert_eq!(
            on_char_insert('(', Some("a"), Some("b"), Some("x"), Some("y")),
            PairAction::InsertPair(')')
        );
    }

    #[test]
    fn test_open_bracket_with_cjk_context() {
        assert_eq!(
            on_char_insert('[', Some("你"), Some("好"), None, None),
            PairAction::InsertPair(']')
        );
    }

    #[test]
    fn test_open_brace_at_line_end() {
        assert_eq!(
            on_char_insert('{', Some("x"), None, None, None),
            PairAction::InsertPair('}')
        );
    }

    // ── Bracket pairs (closing) ────────────────────────────────────

    #[test]
    fn test_close_paren_skip_when_after_matches() {
        assert_eq!(
            on_char_insert(')', None, Some(")"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_close_paren_none_when_after_differs() {
        assert_eq!(
            on_char_insert(')', None, Some("a"), None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_paren_none_when_at_line_end() {
        assert_eq!(
            on_char_insert(')', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_bracket_skip_when_after_matches() {
        assert_eq!(
            on_char_insert(']', None, Some("]"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_close_bracket_none_when_after_differs() {
        assert_eq!(
            on_char_insert(']', None, Some("x"), None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_bracket_none_when_at_line_end() {
        assert_eq!(
            on_char_insert(']', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_brace_skip_when_after_matches() {
        assert_eq!(
            on_char_insert('}', None, Some("}"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_close_brace_none_when_after_differs() {
        assert_eq!(
            on_char_insert('}', None, Some("!"), None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_brace_none_when_at_line_end() {
        assert_eq!(
            on_char_insert('}', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_close_paren_with_cjk_after() {
        assert_eq!(
            on_char_insert(')', None, Some("中"), None, None),
            PairAction::None
        );
    }

    // ── Quote pairs ────────────────────────────────────────────────

    #[test]
    fn test_single_quote_inserts_pair_no_context() {
        assert_eq!(
            on_char_insert('\'', None, None, None, None),
            PairAction::InsertPair('\'')
        );
    }

    #[test]
    fn test_single_quote_skip_when_after_matches() {
        assert_eq!(
            on_char_insert('\'', None, Some("'"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_single_quote_inserts_pair_with_other_after() {
        assert_eq!(
            on_char_insert('\'', Some("a"), Some("b"), None, None),
            PairAction::InsertPair('\'')
        );
    }

    #[test]
    fn test_double_quote_inserts_pair_no_context() {
        assert_eq!(
            on_char_insert('"', None, None, None, None),
            PairAction::InsertPair('"')
        );
    }

    #[test]
    fn test_double_quote_skip_when_after_matches() {
        assert_eq!(
            on_char_insert('"', None, Some("\""), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_double_quote_inserts_pair_with_cjk() {
        assert_eq!(
            on_char_insert('"', Some("中"), None, None, None),
            PairAction::InsertPair('"')
        );
    }

    #[test]
    fn test_single_quote_at_line_end_with_before() {
        assert_eq!(
            on_char_insert('\'', Some("x"), None, None, None),
            PairAction::InsertPair('\'')
        );
    }

    // ── Star pairs ─────────────────────────────────────────────────

    #[test]
    fn test_star_inserts_pair_no_context() {
        assert_eq!(
            on_char_insert('*', None, None, None, None),
            PairAction::InsertPair('*')
        );
    }

    #[test]
    fn test_star_inserts_pair_with_text_context() {
        assert_eq!(
            on_char_insert('*', Some("a"), Some("b"), None, None),
            PairAction::InsertPair('*')
        );
    }

    #[test]
    fn test_star_absorb_at_single_pair() {
        // Cursor at *|* — typing * absorbs to become **|**
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), None, None),
            PairAction::Absorb('*')
        );
    }

    #[test]
    fn test_star_absorb_at_single_pair_with_non_star_outer() {
        // Context: a*|*b — before_2=a, after_2=b, still absorbs
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), Some("a"), Some("b")),
            PairAction::Absorb('*')
        );
    }

    #[test]
    fn test_star_skip_at_double_pair() {
        // Cursor at **|** — typing * skips
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), Some("*"), Some("*")),
            PairAction::Skip
        );
    }

    #[test]
    fn test_star_skip_when_only_after_is_star() {
        // Cursor at a|* — after_1=*, before_1 is not * → skip
        assert_eq!(
            on_char_insert('*', Some("a"), Some("*"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_star_skip_when_at_line_start_after_is_star() {
        assert_eq!(
            on_char_insert('*', None, Some("*"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_star_inserts_pair_with_cjk_context() {
        assert_eq!(
            on_char_insert('*', Some("你"), Some("好"), None, None),
            PairAction::InsertPair('*')
        );
    }

    #[test]
    fn test_star_at_line_start_no_after() {
        assert_eq!(
            on_char_insert('*', None, None, None, None),
            PairAction::InsertPair('*')
        );
    }

    // ── Tilde pairs ────────────────────────────────────────────────

    #[test]
    fn test_tilde_inserts_pair_no_context() {
        assert_eq!(
            on_char_insert('~', None, None, None, None),
            PairAction::InsertPair('~')
        );
    }

    #[test]
    fn test_tilde_inserts_pair_with_text_context() {
        assert_eq!(
            on_char_insert('~', Some("x"), Some("y"), None, None),
            PairAction::InsertPair('~')
        );
    }

    #[test]
    fn test_tilde_absorb_at_single_pair() {
        // Cursor at ~|~ — typing ~ absorbs to become ~~|~~
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), None, None),
            PairAction::Absorb('~')
        );
    }

    #[test]
    fn test_tilde_absorb_at_single_pair_non_tilde_outer() {
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), Some("a"), Some("b")),
            PairAction::Absorb('~')
        );
    }

    #[test]
    fn test_tilde_skip_at_double_pair() {
        // Cursor at ~~|~~ — typing ~ skips
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), Some("~"), Some("~")),
            PairAction::Skip
        );
    }

    #[test]
    fn test_tilde_skip_when_only_after_is_tilde() {
        assert_eq!(
            on_char_insert('~', Some("a"), Some("~"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_tilde_skip_when_at_line_start_after_is_tilde() {
        assert_eq!(
            on_char_insert('~', None, Some("~"), None, None),
            PairAction::Skip
        );
    }

    // ── Backtick pairs ─────────────────────────────────────────────

    #[test]
    fn test_backtick_inserts_pair_no_context() {
        assert_eq!(
            on_char_insert('`', None, None, None, None),
            PairAction::InsertPair('`')
        );
    }

    #[test]
    fn test_backtick_inserts_pair_with_text_context() {
        assert_eq!(
            on_char_insert('`', Some("a"), Some("b"), None, None),
            PairAction::InsertPair('`')
        );
    }

    #[test]
    fn test_backtick_absorb_at_single_pair() {
        // Cursor at `|` — typing ` absorbs to become ``|``
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), None, None),
            PairAction::Absorb('`')
        );
    }

    #[test]
    fn test_backtick_absorb_at_single_pair_non_backtick_outer() {
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), Some("a"), Some("b")),
            PairAction::Absorb('`')
        );
    }

    #[test]
    fn test_backtick_code_block_at_double_pair() {
        // Cursor at ``|`` — typing ` triggers CodeBlock
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), Some("`"), Some("`")),
            PairAction::CodeBlock
        );
    }

    #[test]
    fn test_backtick_skip_when_only_after_is_backtick() {
        assert_eq!(
            on_char_insert('`', Some("a"), Some("`"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_backtick_skip_when_at_line_start_after_is_backtick() {
        assert_eq!(
            on_char_insert('`', None, Some("`"), None, None),
            PairAction::Skip
        );
    }

    #[test]
    fn test_backtick_with_cjk_context() {
        assert_eq!(
            on_char_insert('`', Some("中"), Some("文"), None, None),
            PairAction::InsertPair('`')
        );
    }

    // ── Backtick code block edge cases ─────────────────────────────

    #[test]
    fn test_backtick_code_block_requires_all_four_neighbors() {
        // Missing after_2 — should absorb, not code block
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), Some("`"), None),
            PairAction::Absorb('`')
        );
    }

    #[test]
    fn test_backtick_code_block_missing_before_2() {
        // Missing before_2 — should absorb, not code block
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), None, Some("`")),
            PairAction::Absorb('`')
        );
    }

    #[test]
    fn test_backtick_absorb_non_backtick_outer_before() {
        // before_2=x, after_2=` — should absorb (not code block)
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), Some("x"), Some("`")),
            PairAction::Absorb('`')
        );
    }

    #[test]
    fn test_backtick_absorb_non_backtick_outer_after() {
        // before_2=`, after_2=x — should absorb (not code block)
        assert_eq!(
            on_char_insert('`', Some("`"), Some("`"), Some("`"), Some("x")),
            PairAction::Absorb('`')
        );
    }

    // ── Backspace pair deletion ────────────────────────────────────

    #[test]
    fn test_backspace_delete_parens() {
        assert_eq!(
            on_backspace(Some("("), Some(")")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_brackets() {
        assert_eq!(
            on_backspace(Some("["), Some("]")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_braces() {
        assert_eq!(
            on_backspace(Some("{"), Some("}")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_single_quotes() {
        assert_eq!(
            on_backspace(Some("'"), Some("'")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_double_quotes() {
        assert_eq!(
            on_backspace(Some("\""), Some("\"")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_backticks() {
        assert_eq!(
            on_backspace(Some("`"), Some("`")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_stars() {
        assert_eq!(
            on_backspace(Some("*"), Some("*")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_delete_tildes() {
        assert_eq!(
            on_backspace(Some("~"), Some("~")),
            BackspaceAction::DeletePair
        );
    }

    #[test]
    fn test_backspace_mismatched_paren_bracket() {
        assert_eq!(on_backspace(Some("("), Some("]")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_mismatched_bracket_brace() {
        assert_eq!(on_backspace(Some("["), Some("}")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_mismatched_brace_paren() {
        assert_eq!(on_backspace(Some("{"), Some(")")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_mismatched_quote_types() {
        assert_eq!(on_backspace(Some("'"), Some("\"")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_closing_before_opening_after() {
        // )|( — not a valid pair direction
        assert_eq!(on_backspace(Some(")"), Some("(")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_star_tilde_mismatch() {
        assert_eq!(on_backspace(Some("*"), Some("~")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_backtick_star_mismatch() {
        assert_eq!(on_backspace(Some("`"), Some("*")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_no_context_both_none() {
        assert_eq!(on_backspace(None, None), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_only_before_no_after() {
        assert_eq!(on_backspace(Some("("), None), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_only_after_no_before() {
        assert_eq!(on_backspace(None, Some(")")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_regular_chars() {
        assert_eq!(on_backspace(Some("a"), Some("b")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_cjk_chars() {
        assert_eq!(on_backspace(Some("你"), Some("好")), BackspaceAction::None);
    }

    #[test]
    fn test_backspace_opening_before_regular_after() {
        assert_eq!(on_backspace(Some("("), Some("a")), BackspaceAction::None);
    }

    // ── Opening tag detection ──────────────────────────────────────

    #[test]
    fn test_is_opening_tag_td() {
        assert!(is_opening_tag(":::td"));
    }

    #[test]
    fn test_is_opening_tag_cal() {
        assert!(is_opening_tag(":::cal"));
    }

    #[test]
    fn test_is_opening_tag_note() {
        assert!(is_opening_tag(":::note"));
    }

    #[test]
    fn test_is_opening_tag_with_leading_spaces() {
        assert!(is_opening_tag("  :::td"));
    }

    #[test]
    fn test_is_opening_tag_with_trailing_spaces() {
        assert!(is_opening_tag(":::cal  "));
    }

    #[test]
    fn test_is_opening_tag_with_both_spaces() {
        assert!(is_opening_tag("  :::note  "));
    }

    #[test]
    fn test_is_opening_tag_with_tab() {
        assert!(is_opening_tag("\t:::td\t"));
    }

    #[test]
    fn test_is_opening_tag_closing_tag_false() {
        assert!(!is_opening_tag(":::"));
    }

    #[test]
    fn test_is_opening_tag_unknown_type() {
        assert!(!is_opening_tag(":::other"));
    }

    #[test]
    fn test_is_opening_tag_extra_text_after() {
        assert!(!is_opening_tag(":::td extra text"));
    }

    #[test]
    fn test_is_opening_tag_empty_string() {
        assert!(!is_opening_tag(""));
    }

    #[test]
    fn test_is_opening_tag_only_two_colons() {
        assert!(!is_opening_tag("::"));
    }

    #[test]
    fn test_is_opening_tag_partial_prefix() {
        assert!(!is_opening_tag("::td"));
    }

    #[test]
    fn test_is_opening_tag_case_sensitive_upper() {
        assert!(!is_opening_tag(":::TD"));
    }

    #[test]
    fn test_is_opening_tag_case_sensitive_mixed() {
        assert!(!is_opening_tag(":::Cal"));
    }

    #[test]
    fn test_is_opening_tag_case_sensitive_capitalized() {
        assert!(!is_opening_tag(":::Note"));
    }

    #[test]
    fn test_is_opening_tag_whitespace_only() {
        assert!(!is_opening_tag("   "));
    }

    #[test]
    fn test_is_opening_tag_four_colons() {
        assert!(!is_opening_tag("::::td"));
    }

    #[test]
    fn test_is_opening_tag_no_type_suffix() {
        assert!(!is_opening_tag(":::"));
    }

    // ── Edge cases for on_char_insert ──────────────────────────────

    #[test]
    fn test_regular_char_no_action() {
        assert_eq!(
            on_char_insert('a', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_regular_char_with_full_context() {
        assert_eq!(
            on_char_insert('z', Some("a"), Some("b"), Some("c"), Some("d")),
            PairAction::None
        );
    }

    #[test]
    fn test_digit_no_action() {
        assert_eq!(
            on_char_insert('5', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_space_no_action() {
        assert_eq!(
            on_char_insert(' ', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_cjk_char_no_action() {
        assert_eq!(
            on_char_insert('你', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_bracket_with_cjk_before() {
        assert_eq!(
            on_char_insert('(', Some("中"), None, None, None),
            PairAction::InsertPair(')')
        );
    }

    #[test]
    fn test_brace_with_cjk_after() {
        assert_eq!(
            on_char_insert('{', None, Some("文"), None, None),
            PairAction::InsertPair('}')
        );
    }

    #[test]
    fn test_newline_no_action() {
        assert_eq!(
            on_char_insert('\n', None, None, None, None),
            PairAction::None
        );
    }

    #[test]
    fn test_tab_no_action() {
        assert_eq!(
            on_char_insert('\t', None, None, None, None),
            PairAction::None
        );
    }

    // ── Star/tilde absorb vs skip disambiguation ───────────────────

    #[test]
    fn test_star_absorb_only_before1_after1_star_before2_none() {
        // before_1=*, after_1=*, before_2=None, after_2=None → Absorb
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), None, None),
            PairAction::Absorb('*')
        );
    }

    #[test]
    fn test_tilde_absorb_only_before1_after1_tilde_before2_none() {
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), None, None),
            PairAction::Absorb('~')
        );
    }

    #[test]
    fn test_star_absorb_before2_star_after2_non_star() {
        // before_2=*, after_2=a: b1&&a1 true, b2&&a2 false → Absorb
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), Some("*"), Some("a")),
            PairAction::Absorb('*')
        );
    }

    #[test]
    fn test_star_absorb_before2_non_star_after2_star() {
        // before_2=a, after_2=*: b1&&a1 true, b2&&a2 false → Absorb
        assert_eq!(
            on_char_insert('*', Some("*"), Some("*"), Some("a"), Some("*")),
            PairAction::Absorb('*')
        );
    }

    #[test]
    fn test_tilde_absorb_before2_tilde_after2_non_tilde() {
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), Some("~"), Some("a")),
            PairAction::Absorb('~')
        );
    }

    #[test]
    fn test_tilde_absorb_before2_non_tilde_after2_tilde() {
        assert_eq!(
            on_char_insert('~', Some("~"), Some("~"), Some("a"), Some("~")),
            PairAction::Absorb('~')
        );
    }
}

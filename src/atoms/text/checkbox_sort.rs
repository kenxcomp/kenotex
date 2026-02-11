/// Detect checkbox state from a line.
/// Returns `Some(true)` for checked (`- [x]`/`- [X]`), `Some(false)` for unchecked (`- [ ]`/`- []`),
/// or `None` if the line is not a checkbox.
fn checkbox_state(line: &str) -> Option<bool> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
        Some(true)
    } else if trimmed.starts_with("- [ ]") || trimmed.starts_with("- []") {
        Some(false)
    } else {
        None
    }
}

/// Return the indentation level (number of leading whitespace bytes) of a line.
fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Sort checkboxes within a single paragraph (group of non-blank lines).
/// Checked items (`- [x]`) are moved to the bottom; unchecked remain on top.
/// Sub-items (lines indented more than their parent checkbox) stay grouped with their parent.
/// Non-checkbox lines retain their original positions.
pub fn sort_paragraph_checkboxes(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Build groups: each top-level checkbox (with sub-items) or non-checkbox line forms a group.
    // Tuple: (is_checkbox, is_checked, group_index, lines)
    let mut groups: Vec<(bool, bool, usize, Vec<String>)> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some(checked) = checkbox_state(&lines[i]) {
            let parent_indent = indent_level(&lines[i]);
            let mut group_lines = vec![lines[i].clone()];
            i += 1;
            // Collect sub-items: lines indented more than the parent checkbox
            while i < lines.len()
                && indent_level(&lines[i]) > parent_indent
                && !lines[i].trim().is_empty()
            {
                group_lines.push(lines[i].clone());
                i += 1;
            }
            let idx = groups.len();
            groups.push((true, checked, idx, group_lines));
        } else {
            let idx = groups.len();
            groups.push((false, false, idx, vec![lines[i].clone()]));
            i += 1;
        }
    }

    // Track which group slots are checkboxes vs non-checkboxes
    let mut non_cb_positions: Vec<(usize, Vec<String>)> = Vec::new();
    let mut cb_slot_indices: Vec<usize> = Vec::new();

    for (idx, (is_cb, _, _, group_lines)) in groups.iter().enumerate() {
        if *is_cb {
            cb_slot_indices.push(idx);
        } else {
            non_cb_positions.push((idx, group_lines.clone()));
        }
    }

    // Sort checkbox groups: unchecked first, checked last (stable order within each)
    let mut sorted_cb: Vec<&(bool, bool, usize, Vec<String>)> =
        groups.iter().filter(|(is_cb, _, _, _)| *is_cb).collect();
    sorted_cb.sort_by_key(|(_, is_checked, _, _)| *is_checked);

    // Rebuild groups in order: non-checkbox groups stay in place, checkbox slots get sorted groups
    let mut final_groups: Vec<Vec<String>> = vec![Vec::new(); groups.len()];
    for (pos, group) in &non_cb_positions {
        final_groups[*pos] = group.clone();
    }
    for (i, cb_slot) in cb_slot_indices.iter().enumerate() {
        final_groups[*cb_slot] = sorted_cb[i].3.clone();
    }

    let mut result: Vec<String> = Vec::new();
    for group in final_groups {
        result.extend(group);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_state_unchecked() {
        assert_eq!(checkbox_state("- [ ] task"), Some(false));
        assert_eq!(checkbox_state("  - [ ] indented"), Some(false));
        assert_eq!(checkbox_state("- [] empty"), Some(false));
    }

    #[test]
    fn test_checkbox_state_checked() {
        assert_eq!(checkbox_state("- [x] done"), Some(true));
        assert_eq!(checkbox_state("- [X] done"), Some(true));
        assert_eq!(checkbox_state("  - [x] indented done"), Some(true));
    }

    #[test]
    fn test_checkbox_state_non_checkbox() {
        assert_eq!(checkbox_state("hello"), None);
        assert_eq!(checkbox_state("- regular list"), None);
        assert_eq!(checkbox_state("* bullet"), None);
    }

    #[test]
    fn test_sort_basic() {
        let lines: Vec<String> = vec![
            "- [x] done task".into(),
            "- [ ] todo task".into(),
            "- [x] another done".into(),
            "- [ ] another todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] todo task",
            "- [ ] another todo",
            "- [x] done task",
            "- [x] another done",
        ]);
    }

    #[test]
    fn test_sort_already_sorted() {
        let lines: Vec<String> = vec![
            "- [ ] todo".into(),
            "- [x] done".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["- [ ] todo", "- [x] done"]);
    }

    #[test]
    fn test_sort_with_sub_items() {
        let lines: Vec<String> = vec![
            "- [x] parent done".into(),
            "  - sub item 1".into(),
            "  - sub item 2".into(),
            "- [ ] parent todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] parent todo",
            "- [x] parent done",
            "  - sub item 1",
            "  - sub item 2",
        ]);
    }

    #[test]
    fn test_sort_non_checkbox_lines_pinned() {
        let lines: Vec<String> = vec![
            "# Header".into(),
            "- [x] done".into(),
            "- [ ] todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "# Header",
            "- [ ] todo",
            "- [x] done",
        ]);
    }

    #[test]
    fn test_sort_empty() {
        let result = sort_paragraph_checkboxes(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sort_no_checkboxes() {
        let lines: Vec<String> = vec![
            "just text".into(),
            "more text".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["just text", "more text"]);
    }

    #[test]
    fn test_sort_all_unchecked() {
        let lines: Vec<String> = vec![
            "- [ ] a".into(),
            "- [ ] b".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["- [ ] a", "- [ ] b"]);
    }

    #[test]
    fn test_sort_all_checked() {
        let lines: Vec<String> = vec![
            "- [x] a".into(),
            "- [x] b".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["- [x] a", "- [x] b"]);
    }

    #[test]
    fn test_sort_mixed_with_header_and_sub_items() {
        let lines: Vec<String> = vec![
            "## Tasks".into(),
            "- [x] completed".into(),
            "  - detail".into(),
            "- [ ] pending".into(),
            "- [x] also done".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "## Tasks",
            "- [ ] pending",
            "- [x] completed",
            "  - detail",
            "- [x] also done",
        ]);
    }

    // ── CJK content ─────────────────────────────────────────────────

    #[test]
    fn test_sort_cjk_content() {
        let lines: Vec<String> = vec![
            "- [x] 买牛奶".into(),
            "- [ ] 遛狗".into(),
            "- [x] 写代码".into(),
            "- [ ] 读书".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] 遛狗",
            "- [ ] 读书",
            "- [x] 买牛奶",
            "- [x] 写代码",
        ]);
    }

    #[test]
    fn test_sort_cjk_with_sub_items() {
        let lines: Vec<String> = vec![
            "- [x] 完成任务".into(),
            "  - 子任务一".into(),
            "  - 子任务二".into(),
            "- [ ] 待办事项".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] 待办事项",
            "- [x] 完成任务",
            "  - 子任务一",
            "  - 子任务二",
        ]);
    }

    // ── Stable order ────────────────────────────────────────────────

    #[test]
    fn test_sort_stable_order_unchecked() {
        // Relative order within unchecked items must be preserved
        let lines: Vec<String> = vec![
            "- [ ] alpha".into(),
            "- [x] done".into(),
            "- [ ] beta".into(),
            "- [ ] gamma".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        // alpha, beta, gamma should stay in their original relative order
        assert_eq!(result, vec![
            "- [ ] alpha",
            "- [ ] beta",
            "- [ ] gamma",
            "- [x] done",
        ]);
    }

    #[test]
    fn test_sort_stable_order_checked() {
        // Relative order within checked items must be preserved
        let lines: Vec<String> = vec![
            "- [x] first done".into(),
            "- [ ] todo".into(),
            "- [x] second done".into(),
            "- [x] third done".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] todo",
            "- [x] first done",
            "- [x] second done",
            "- [x] third done",
        ]);
    }

    // ── Case insensitive X ──────────────────────────────────────────

    #[test]
    fn test_sort_case_insensitive_x() {
        let lines: Vec<String> = vec![
            "- [X] uppercase done".into(),
            "- [ ] todo".into(),
            "- [x] lowercase done".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] todo",
            "- [X] uppercase done",
            "- [x] lowercase done",
        ]);
    }

    // ── Non-checkbox lines retain positions ──────────────────────────

    #[test]
    fn test_sort_non_checkbox_between_checkboxes() {
        let lines: Vec<String> = vec![
            "- [x] done".into(),
            "Some plain text".into(),
            "- [ ] todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] todo",
            "Some plain text",
            "- [x] done",
        ]);
    }

    #[test]
    fn test_sort_multiple_non_checkbox_lines() {
        let lines: Vec<String> = vec![
            "# Title".into(),
            "- [x] done 1".into(),
            "Description text".into(),
            "- [ ] todo 1".into(),
            "- [x] done 2".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "# Title",
            "- [ ] todo 1",
            "Description text",
            "- [x] done 1",
            "- [x] done 2",
        ]);
    }

    // ── Sub-items edge cases ────────────────────────────────────────

    #[test]
    fn test_sort_nested_sub_items_stay_grouped() {
        let lines: Vec<String> = vec![
            "- [x] parent 1".into(),
            "  - child 1a".into(),
            "  - child 1b".into(),
            "- [ ] parent 2".into(),
            "  - child 2a".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] parent 2",
            "  - child 2a",
            "- [x] parent 1",
            "  - child 1a",
            "  - child 1b",
        ]);
    }

    #[test]
    fn test_sort_deeply_indented_sub_items() {
        let lines: Vec<String> = vec![
            "- [x] done task".into(),
            "  - sub level 1".into(),
            "    - sub level 2".into(),
            "- [ ] todo task".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [ ] todo task",
            "- [x] done task",
            "  - sub level 1",
            "    - sub level 2",
        ]);
    }

    // ── Mixed checkbox formats ──────────────────────────────────────

    #[test]
    fn test_sort_empty_bracket_checkbox() {
        // `- []` should also be detected as unchecked
        let lines: Vec<String> = vec![
            "- [x] done".into(),
            "- [] also todo".into(),
            "- [ ] todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "- [] also todo",
            "- [ ] todo",
            "- [x] done",
        ]);
    }

    // ── Single item ─────────────────────────────────────────────────

    #[test]
    fn test_sort_single_checkbox() {
        let lines: Vec<String> = vec!["- [ ] only one".into()];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["- [ ] only one"]);
    }

    #[test]
    fn test_sort_single_non_checkbox() {
        let lines: Vec<String> = vec!["plain text".into()];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec!["plain text"]);
    }

    // ── Indented checkboxes ─────────────────────────────────────────

    #[test]
    fn test_sort_indented_checkboxes() {
        let lines: Vec<String> = vec![
            "  - [x] indented done".into(),
            "  - [ ] indented todo".into(),
        ];
        let result = sort_paragraph_checkboxes(&lines);
        assert_eq!(result, vec![
            "  - [ ] indented todo",
            "  - [x] indented done",
        ]);
    }
}

//! Comprehensive CJK/wide-character edge case tests.
//!
//! These tests cover visual block selection, cursor movement, soft-wrap,
//! block insert, and delete operations with mixed ASCII and CJK text.
//!
//! The core issue: Visual Block mode operations in VisualMode (render_data,
//! delete_selection, yank_selection, prepare_insert_*) use grapheme column
//! indices for block bounds. But for a visually rectangular block across lines
//! with different character widths (CJK vs ASCII), the bounds must be defined
//! in display columns. The same grapheme index maps to different display
//! columns on different lines.

use kenotex::atoms::widgets::wrap_calc;
use kenotex::molecules::editor::{RenderSelection, TextBuffer, VisualMode, VisualType};

// ============================================================================
// Group A: Visual Block Selection with Display-Column Alignment
// ============================================================================

/// When selecting a visual block across lines with mixed ASCII and CJK,
/// the block should be defined by display columns, not grapheme indices.
///
/// Line 0: "Hello" (all ASCII: grapheme idx = display col)
/// Line 1: "你好world" (CJK: "你" cols 0-1, "好" cols 2-3, "w" col 4...)
///
/// Scenario: User enters visual block at (0,2) "l" (display col 2), then moves
/// down. The event dispatcher maps display col 2 on line 1 to grapheme 1 ("好").
/// So cursor = (1, 1).
///
/// render_data gets anchor=(0,2), cursor=(1,1).
/// Current behavior: left_col = min(2,1) = 1, right_col = max(2,1) = 2
///   → Line 0: graphemes 1-2 = "el" (display cols 1-2)
///   → Line 1: graphemes 1-2 = "好w" (display cols 2-4)
///   This is NOT a visual rectangle!
///
/// Correct behavior: The block should be defined by display columns.
/// anchor display = 2, cursor display = 2 → display range [2, 3] (好 is width 2)
///   → Line 0: display cols 2-3 = "ll" (graphemes 2-3)
///   → Line 1: display cols 2-3 = "好" (grapheme 1, width 2)
///   This IS a visual rectangle.
#[test]
fn test_block_render_data_uses_display_columns() {
    let buffer = TextBuffer::from_string("Hello\n你好world");

    // Simulate: anchor at (0,2) display col 2, cursor moved down to line 1
    let anchor = (0, 2); // "l" at display col 2
    let cursor_grapheme = buffer.grapheme_at_display_col(1, 2); // = 1 ("好")
    let cursor = (1, cursor_grapheme);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let render = visual.render_data(&buffer, cursor);

    match render {
        RenderSelection::BlockRegion {
            top_row,
            bottom_row,
            left_col,
            right_col,
        } => {
            assert_eq!(top_row, 0);
            assert_eq!(bottom_row, 1);
            // The block region should represent display columns, not grapheme indices.
            // Both anchor and cursor are at display col 2, but the char at cursor
            // (好) has width 2, so the block extends to display col 3.
            assert_eq!(
                left_col, 2,
                "left_col should be display col 2 (bug: currently min(2,1)=1)"
            );
            assert_eq!(
                right_col, 3,
                "right_col should be display col 3 (bug: currently max(2,1)=2)"
            );
        }
        _ => panic!("Expected BlockRegion for Block visual mode"),
    }
}

/// Block selection with CJK anchor and ASCII cursor — display columns must align.
#[test]
fn test_block_render_data_cjk_anchor_ascii_cursor() {
    let buffer = TextBuffer::from_string("你好世界\nabcdefgh");

    // anchor (0,1) = "好" at display cols 2-3
    // cursor (1,3) = "d" at display col 3
    let anchor = (0, 1);
    let cursor = (1, 3);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let render = visual.render_data(&buffer, cursor);

    match render {
        RenderSelection::BlockRegion {
            left_col,
            right_col,
            ..
        } => {
            // anchor "好" starts at display 2, cursor "d" at display 3
            // Block: display cols [2, 3]
            assert_eq!(
                left_col, 2,
                "left should be display col 2 (bug: min(1,3)=1)"
            );
            assert_eq!(
                right_col, 3,
                "right should be display col 3 (bug: max(1,3)=3)"
            );
        }
        _ => panic!("Expected BlockRegion"),
    }
}

/// Block selection where display col range spans mid-wide-char.
#[test]
fn test_block_render_data_mid_wide_char() {
    let buffer = TextBuffer::from_string("ab\n你好");

    // anchor (0,1) = "b" at display col 1
    // cursor (1,0) = "你" at display col 0
    let anchor = (0, 1);
    let cursor = (1, 0);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let render = visual.render_data(&buffer, cursor);

    match render {
        RenderSelection::BlockRegion {
            left_col,
            right_col,
            ..
        } => {
            // anchor display = 1, cursor display = 0
            // cursor "你" has width 2, so extends to display col 1
            // Block: display cols [0, 1]
            assert_eq!(left_col, 0, "left should be display col 0");
            assert_eq!(
                right_col, 1,
                "right should be display col 1 (end of 你 or b)"
            );
        }
        _ => panic!("Expected BlockRegion"),
    }
}

// ============================================================================
// Group B: Cursor Movement Across Mixed ASCII + CJK Lines
// ============================================================================

/// display_col_at correctly maps grapheme indices to display columns for CJK.
#[test]
fn test_cursor_display_col_cjk_movement() {
    let buffer = TextBuffer::from_string("你好世界");
    assert_eq!(buffer.display_col_at(0, 0), 0);
    assert_eq!(buffer.display_col_at(0, 1), 2);
    assert_eq!(buffer.display_col_at(0, 2), 4);
    assert_eq!(buffer.display_col_at(0, 3), 6);
}

/// grapheme_at_display_col snaps to start of wide char when target is mid-char.
#[test]
fn test_grapheme_at_display_col_mid_wide_char() {
    let buffer = TextBuffer::from_string("你好世界");
    assert_eq!(buffer.grapheme_at_display_col(0, 0), 0);
    assert_eq!(
        buffer.grapheme_at_display_col(0, 1),
        0,
        "mid-你 should snap to grapheme 0"
    );
    assert_eq!(buffer.grapheme_at_display_col(0, 2), 1);
    assert_eq!(
        buffer.grapheme_at_display_col(0, 3),
        1,
        "mid-好 should snap to grapheme 1"
    );
}

/// Mixed ASCII + CJK display columns.
#[test]
fn test_cursor_display_col_mixed_line() {
    let buffer = TextBuffer::from_string("AB中文EF");
    assert_eq!(buffer.display_col_at(0, 0), 0); // A
    assert_eq!(buffer.display_col_at(0, 1), 1); // B
    assert_eq!(buffer.display_col_at(0, 2), 2); // 中
    assert_eq!(buffer.display_col_at(0, 3), 4); // 文
    assert_eq!(buffer.display_col_at(0, 4), 6); // E
    assert_eq!(buffer.display_col_at(0, 5), 7); // F
}

/// Vertical movement preserves display column across mixed lines.
#[test]
fn test_vertical_movement_maintains_display_col() {
    let buffer = TextBuffer::from_string("Hello\n你好world");
    let display_col = buffer.display_col_at(0, 4); // "o" at display col 4
    assert_eq!(display_col, 4);
    let target_grapheme = buffer.grapheme_at_display_col(1, display_col);
    // "你"(0-1) "好"(2-3) "w"(4) → grapheme 2
    assert_eq!(
        target_grapheme, 2,
        "display col 4 on '你好world' should be grapheme 2 ('w')"
    );
}

// ============================================================================
// Group C: Soft-Wrap Never Splits a Wide Character
// ============================================================================

/// A wide char that would be split at boundary wraps to the next line.
#[test]
fn test_soft_wrap_no_split_wide_char_at_boundary() {
    assert_eq!(wrap_calc::display_rows_for_line("ab你", 3), 2);
    assert_eq!(wrap_calc::display_rows_for_line("ab你", 4), 1);
}

/// Cursor position after a wide char wraps.
#[test]
fn test_cursor_position_after_wide_char_wrap() {
    let lines = vec!["ab你好".to_string()];
    // Width 3: "ab"(2 cols) row 0, "你"(2) wraps→row 1, "好"(2) wraps→row 2
    // Each CJK char needs 2 cols but only 1 col remains after the previous, so each wraps.
    let vpos = wrap_calc::visual_cursor_position(&lines, 0, 3, 3); // grapheme 3 = "好"
    assert_eq!(vpos.wrap_row, 2, "好 should be on wrap row 2");
    assert_eq!(vpos.col, 0, "好 should be at display col 0 on its wrap row");
}

/// CJK sequence with odd width wraps correctly.
#[test]
fn test_soft_wrap_cjk_sequence_odd_width() {
    // Each CJK char width 2, width 3: each char on its own row
    assert_eq!(wrap_calc::display_rows_for_line("你好世界", 3), 4);
}

/// Mixed content soft-wrap.
#[test]
fn test_soft_wrap_mixed_content() {
    // "Hi你好AB" = H(1)+i(1)+你(2)+好(2)+A(1)+B(1) = 8
    // Width 5: "Hi你"(4), "好AB" wraps: 好(2)+A(1)+B(1)=4 → 2 rows
    assert_eq!(wrap_calc::display_rows_for_line("Hi你好AB", 5), 2);
}

// ============================================================================
// Group D: Block Insert (A/I) Across CJK Lines
// ============================================================================

/// Block insert at start (I) should produce display-column-aligned positions.
///
/// Line 0: "Hello"
/// Line 1: "你好world"
///
/// Visual block with anchor (0,2) and cursor (1,1) — both at display col 2.
/// Insert positions should be at display col 2 on each line:
/// - Line 0: grapheme 2 (display col 2)
/// - Line 1: grapheme 1 (好 at display col 2)
#[test]
fn test_block_insert_start_display_aligned() {
    let mut buffer = TextBuffer::from_string("Hello\n你好world");

    let anchor = (0, 2);
    let cursor_grapheme = buffer.grapheme_at_display_col(1, 2); // = 1
    let cursor = (1, cursor_grapheme);

    let mut visual = VisualMode::new(VisualType::Block, anchor);
    let positions = visual.prepare_insert_start(&mut buffer, cursor);

    assert_eq!(positions.len(), 2);
    // Left col should be min of display-aligned grapheme positions
    // anchor grapheme 2 on line 0 = display col 2
    // cursor grapheme 1 on line 1 = display col 2
    // Both at display col 2, so positions should be:
    // Line 0: grapheme at display col 2 = 2
    // Line 1: grapheme at display col 2 = 1
    assert_eq!(
        positions[0],
        (0, 2),
        "Line 0: insert at grapheme 2 (display col 2)"
    );
    assert_eq!(
        positions[1],
        (1, 1),
        "Line 1: insert at grapheme 1 (好 at display col 2)"
    );
}

/// Block insert at end (A) should produce display-column-aligned positions.
#[test]
fn test_block_insert_end_display_aligned() {
    let mut buffer = TextBuffer::from_string("Hello\n你好world");

    // anchor at (0, 4) = "o" display col 4
    // cursor at display col 4 on line 1 = grapheme 2 = "w"
    let anchor = (0, 4);
    let cursor_grapheme = buffer.grapheme_at_display_col(1, 4); // = 2 ("w")
    let cursor = (1, cursor_grapheme);

    let mut visual = VisualMode::new(VisualType::Block, anchor);
    let positions = visual.prepare_insert_end(&mut buffer, cursor);

    assert_eq!(positions.len(), 2);
    // Right col should be max of display-aligned grapheme positions + 1
    // anchor grapheme 4 on line 0 = display col 4 → insert after = grapheme 5
    // cursor grapheme 2 on line 1 = display col 4 → insert after = grapheme 3
    assert_eq!(
        positions[0],
        (0, 5),
        "Line 0: insert after grapheme 4 (past 'o')"
    );
    assert_eq!(
        positions[1],
        (1, 3),
        "Line 1: insert after grapheme 2 (past 'w')"
    );
}

// ============================================================================
// Group E: Delete/Yank in Visual Block Mode with CJK
// ============================================================================

/// Block delete across mixed lines should delete display-column-aligned content.
///
/// Line 0: "Hello world"
/// Line 1: "你好world!!"
///
/// anchor (0,3) "l" display 3, cursor (1,1) "好" display 2.
/// Display range: [2, 3].
/// Line 0: display 2-3 = "ll" (graphemes 2-3)
/// Line 1: display 2-3 = "好" (grapheme 1, width 2)
#[test]
fn test_block_delete_display_aligned() {
    let mut buffer = TextBuffer::from_string("Hello world\n你好world!!");

    let anchor = (0, 3); // "l" at display col 3
    let cursor_grapheme = buffer.grapheme_at_display_col(1, 2); // "好" = grapheme 1
    let cursor = (1, cursor_grapheme);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let deleted = visual.delete_selection(&mut buffer, cursor);

    let lines = buffer.content();
    // Current buggy behavior: left_col = min(3,1)=1, right_col = max(3,1)=3
    // Line 0: deletes graphemes 1-3 = "ell"
    // Line 1: deletes graphemes 1-3 = "好w"
    // Correct behavior: display cols [2,3]
    // Line 0: display 2-3 = graphemes 2-3 = "ll" → "Heo world"
    // Line 1: display 2-3 = grapheme 1 "好" → "你world!!"
    assert_eq!(
        lines[0], "Heo world",
        "Line 0: 'll' (display cols 2-3) should be deleted"
    );
    assert_eq!(
        lines[1], "你world!!",
        "Line 1: '好' (display cols 2-3) should be deleted"
    );
}

/// Block yank across mixed lines should yank display-column-aligned content.
#[test]
fn test_block_yank_display_aligned() {
    let buffer = TextBuffer::from_string("abcdef\n你好世界");

    // anchor (0,2) = "c" display col 2
    // cursor (1,1) = "好" display col 2
    // Both at display col 2. "c" width 1, "好" width 2.
    // Block display cols [2, 3] (好 extends to 3).
    // Line 0: display 2-3 = "cd" (graphemes 2-3)
    // Line 1: display 2-3 = "好" (grapheme 1)
    let anchor = (0, 2);
    let cursor = (1, 1);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let yanked = visual.yank_selection(&buffer, cursor);

    let parts: Vec<&str> = yanked.split('\n').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(
        parts[0], "cd",
        "Line 0: should yank 'cd' (display cols 2-3)"
    );
    assert_eq!(
        parts[1], "好",
        "Line 1: should yank '好' (display cols 2-3)"
    );
}

/// Delete should preserve correct cursor position with CJK.
#[test]
fn test_block_delete_cursor_position_with_cjk() {
    let mut buffer = TextBuffer::from_string("你好世界\nabcdefgh");

    // Select block covering display cols 2-3 ("好" on line 0, "cd" on line 1)
    let anchor = (0, 1); // "好" at display col 2
    let cursor = (1, 3); // "d" at display col 3

    let visual = VisualMode::new(VisualType::Block, anchor);
    let _deleted = visual.delete_selection(&mut buffer, cursor);

    let (cursor_row, cursor_col) = buffer.cursor_position();
    assert_eq!(cursor_row, 0, "cursor should be on first selected row");
    let display_col = buffer.display_col_at(cursor_row, cursor_col);
    assert_eq!(
        display_col, 2,
        "cursor display col should be 2 after delete"
    );
}

/// Block delete where right edge overlaps a wide char includes the full char.
#[test]
fn test_block_delete_right_edge_mid_wide_char() {
    let mut buffer = TextBuffer::from_string("abcde\n你好世界");

    // anchor (0,2) = "c" display col 2
    // cursor (1,0) = "你" display col 0
    // Display range: [0, 2]
    // Line 0: display 0-2 = "abc"
    // Line 1: display 0-2: "你"(0-1) fully in, "好"(2-3) starts at 2 → included
    //   So "你好" should be deleted
    let anchor = (0, 2);
    let cursor = (1, 0);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let deleted = visual.delete_selection(&mut buffer, cursor);

    let lines = buffer.content();
    assert_eq!(lines[0], "de", "Line 0: 'abc' deleted");
    assert_eq!(
        lines[1], "世界",
        "Line 1: '你好' deleted (好 overlaps right edge)"
    );
    assert!(deleted.contains("abc"), "deleted should contain 'abc'");
    assert!(
        deleted.contains("你好"),
        "deleted should contain '你好' (好 overlaps boundary)"
    );
}

// ============================================================================
// Group F: Edge Cases — All CJK Lines
// ============================================================================

/// Block selection on lines that are entirely CJK characters.
#[test]
fn test_block_selection_all_cjk_lines() {
    let buffer = TextBuffer::from_string("你好世界\n天地玄黄");

    // anchor (0,1) = "好" display 2, cursor (1,2) = "玄" display 4
    let anchor = (0, 1);
    let cursor = (1, 2);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let render = visual.render_data(&buffer, cursor);

    match render {
        RenderSelection::BlockRegion {
            left_col,
            right_col,
            ..
        } => {
            // "好" display 2 (width 2→cols 2-3), "玄" display 4 (width 2→cols 4-5)
            // Block: display cols [2, 5]
            assert_eq!(left_col, 2, "left should be display col 2");
            assert_eq!(
                right_col, 5,
                "right should be display col 5 (end of 玄)"
            );
        }
        _ => panic!("Expected BlockRegion"),
    }
}

/// Yank from all-CJK block selection.
#[test]
fn test_block_yank_all_cjk() {
    let buffer = TextBuffer::from_string("你好世界\n天地玄黄");

    // anchor (0,1) = "好" display 2-3
    // cursor (1,1) = "地" display 2-3
    // Block: display cols [2, 3]
    let anchor = (0, 1);
    let cursor = (1, 1);

    let visual = VisualMode::new(VisualType::Block, anchor);
    let yanked = visual.yank_selection(&buffer, cursor);

    let parts: Vec<&str> = yanked.split('\n').collect();
    assert_eq!(parts[0], "好", "should yank 好 from line 0");
    assert_eq!(parts[1], "地", "should yank 地 from line 1");
}

// ============================================================================
// Group G: Wrap Calc with CJK in Visual Positions
// ============================================================================

/// visual_positions_in_range reports correct display widths for CJK chars.
#[test]
fn test_visual_positions_cjk_chars() {
    let positions = wrap_calc::visual_positions_in_range("A你B", 0, 3, 10);
    assert_eq!(positions.len(), 3);
    assert_eq!(positions[0], (0, 0, 1), "A: row 0, col 0, width 1");
    assert_eq!(positions[1], (0, 1, 2), "你: row 0, col 1, width 2");
    assert_eq!(positions[2], (0, 3, 1), "B: row 0, col 3, width 1");
}

/// CJK char at wrap boundary moves to next row.
#[test]
fn test_visual_positions_cjk_wrap_boundary() {
    let positions = wrap_calc::visual_positions_in_range("abc你", 0, 4, 4);
    assert_eq!(positions.len(), 4);
    assert_eq!(positions[0], (0, 0, 1));
    assert_eq!(positions[1], (0, 1, 1));
    assert_eq!(positions[2], (0, 2, 1));
    assert_eq!(positions[3], (1, 0, 2), "你 should wrap to row 1");
}

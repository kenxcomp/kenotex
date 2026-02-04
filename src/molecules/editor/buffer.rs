use unicode_segmentation::UnicodeSegmentation;

use super::list_prefix;
use super::vim_mode::Motion;

const MAX_UNDO_LEVELS: usize = 50;

#[derive(Debug, Clone)]
struct BufferSnapshot {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

#[derive(Debug, Clone, Default)]
struct UndoHistory {
    undo_stack: Vec<BufferSnapshot>,
    redo_stack: Vec<BufferSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    history: UndoHistory,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            history: UndoHistory::default(),
        }
    }

    pub fn from_string(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(String::from).collect()
        };

        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            history: UndoHistory::default(),
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn content(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.lines.len().saturating_sub(1));
        let line_len = self.current_line_len();
        self.cursor_col = col.min(line_len);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map(|l| l.graphemes(true).count())
            .unwrap_or(0)
    }

    fn current_line(&self) -> &str {
        self.lines.get(self.cursor_row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
            return;
        }

        if self.cursor_row >= self.lines.len() {
            self.lines.push(String::new());
        }

        let line = &mut self.lines[self.cursor_row];
        let graphemes: Vec<&str> = line.graphemes(true).collect();

        let insert_pos = self.cursor_col.min(graphemes.len());
        let new_line: String = graphemes[..insert_pos]
            .iter()
            .chain(std::iter::once(&c.to_string().as_str()))
            .chain(graphemes[insert_pos..].iter())
            .copied()
            .collect();

        self.lines[self.cursor_row] = new_line;
        self.cursor_col += 1;
    }

    pub fn insert_tab(&mut self, tab_width: u8) {
        for _ in 0..tab_width {
            self.insert_char(' ');
        }
    }

    /// Insert a string of text at the cursor, handling newlines by splitting lines.
    /// Used for bracketed paste in Insert mode.
    pub fn insert_text(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.insert_newline();
            } else if c != '\r' {
                self.insert_char(c);
            }
        }
    }

    pub fn indent_line(&mut self, tab_width: u8) {
        let spaces: String = " ".repeat(tab_width as usize);
        self.lines[self.cursor_row] = format!("{}{}", spaces, self.lines[self.cursor_row]);
        self.cursor_col += tab_width as usize;
    }

    pub fn dedent_line(&mut self, tab_width: u8) {
        let line = &self.lines[self.cursor_row];
        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        let remove = leading_spaces.min(tab_width as usize);
        if remove > 0 {
            self.lines[self.cursor_row] = self.lines[self.cursor_row][remove..].to_string();
            self.cursor_col = self.cursor_col.saturating_sub(remove);
        }
    }

    pub fn indent_lines(&mut self, start_row: usize, end_row: usize, tab_width: u8) {
        let spaces: String = " ".repeat(tab_width as usize);
        for row in start_row..=end_row.min(self.lines.len().saturating_sub(1)) {
            self.lines[row] = format!("{}{}", spaces, self.lines[row]);
        }
        self.cursor_col += tab_width as usize;
    }

    pub fn dedent_lines(&mut self, start_row: usize, end_row: usize, tab_width: u8) {
        for row in start_row..=end_row.min(self.lines.len().saturating_sub(1)) {
            let leading = self.lines[row].chars().take_while(|c| *c == ' ').count();
            let remove = leading.min(tab_width as usize);
            if remove > 0 {
                self.lines[row] = self.lines[row][remove..].to_string();
            }
        }
        self.cursor_col = self.cursor_col.min(
            self.lines[self.cursor_row]
                .graphemes(true)
                .count(),
        );
    }

    pub fn insert_newline(&mut self) {
        let line = &self.lines[self.cursor_row];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let split_pos = self.cursor_col.min(graphemes.len());

        let before: String = graphemes[..split_pos].iter().copied().collect();
        let after: String = graphemes[split_pos..].iter().copied().collect();

        self.lines[self.cursor_row] = before;
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, after);
        self.cursor_col = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let graphemes: Vec<&str> = line.graphemes(true).collect();

            if self.cursor_col <= graphemes.len() {
                let new_line: String = graphemes[..self.cursor_col - 1]
                    .iter()
                    .chain(graphemes[self.cursor_col..].iter())
                    .copied()
                    .collect();
                self.lines[self.cursor_row] = new_line;
                self.cursor_col -= 1;
            }
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].graphemes(true).count();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    pub fn delete_char(&mut self) {
        let line_len = self.current_line_len();
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_row];
            let graphemes: Vec<&str> = line.graphemes(true).collect();

            let new_line: String = graphemes[..self.cursor_col]
                .iter()
                .chain(graphemes[self.cursor_col + 1..].iter())
                .copied()
                .collect();
            self.lines[self.cursor_row] = new_line;
        } else if self.cursor_row < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.current_line_len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.current_line_len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.current_line_len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.cursor_col = self.current_line_len();
    }

    pub fn move_to_first_line(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn move_to_last_line(&mut self) {
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = 0;
    }

    pub fn move_word_forward(&mut self) {
        let line = self.current_line();
        let graphemes: Vec<&str> = line.graphemes(true).collect();

        let mut pos = self.cursor_col;

        // Skip current word (non-whitespace) or whitespace
        if pos < graphemes.len() && !graphemes[pos].chars().all(|c| c.is_whitespace()) {
            while pos < graphemes.len() && !graphemes[pos].chars().all(|c| c.is_whitespace()) {
                pos += 1;
            }
        }

        // Skip whitespace to reach start of next word
        while pos < graphemes.len() && graphemes[pos].chars().all(|c| c.is_whitespace()) {
            pos += 1;
        }

        if pos >= graphemes.len() && self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col = pos;
        }
    }

    pub fn move_word_backward(&mut self) {
        if self.cursor_col == 0 && self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
            return;
        }

        let line = self.current_line();
        let graphemes: Vec<&str> = line.graphemes(true).collect();

        let mut pos = self.cursor_col.saturating_sub(1);

        while pos > 0 && graphemes[pos].chars().all(|c| c.is_whitespace()) {
            pos -= 1;
        }

        while pos > 0 && !graphemes[pos - 1].chars().all(|c| c.is_whitespace()) {
            pos -= 1;
        }

        self.cursor_col = pos;
    }

    pub fn delete_line(&mut self) {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
        } else {
            self.lines[0].clear();
        }
        self.cursor_col = 0;
    }

    pub fn insert_line_below(&mut self) {
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, String::new());
        self.cursor_col = 0;
    }

    pub fn insert_line_above(&mut self) {
        self.lines.insert(self.cursor_row, String::new());
        self.cursor_col = 0;
    }

    /// Public accessor for the current line content.
    pub fn current_line_content(&self) -> &str {
        self.current_line()
    }

    /// Insert a new line below the current one with a given prefix.
    /// Cursor moves to the new line, positioned at the end of the prefix.
    pub fn insert_line_below_with_prefix(&mut self, prefix: &str) {
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, prefix.to_string());
        self.cursor_col = prefix.graphemes(true).count();
    }

    /// Split the current line at the cursor, prepend the given prefix to the
    /// new (lower) line. Cursor moves to the new line at the end of the prefix.
    pub fn insert_newline_with_prefix(&mut self, prefix: &str) {
        let line = &self.lines[self.cursor_row];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let split_pos = self.cursor_col.min(graphemes.len());

        let before: String = graphemes[..split_pos].iter().copied().collect();
        let after: String = graphemes[split_pos..].iter().copied().collect();

        self.lines[self.cursor_row] = before;
        self.cursor_row += 1;
        let new_line = format!("{}{}", prefix, after);
        self.lines.insert(self.cursor_row, new_line);
        self.cursor_col = prefix.graphemes(true).count();
    }

    /// Clear the current line (replace with empty string), set cursor_col = 0.
    pub fn clear_current_line(&mut self) {
        self.lines[self.cursor_row] = String::new();
        self.cursor_col = 0;
    }

    /// Insert a `- [ ] ` checkbox prefix on the current line.
    /// Does nothing if a checkbox prefix already exists.
    pub fn insert_checkbox(&mut self) {
        let line = self.current_line().to_string();
        if let Some(new_line) = list_prefix::insert_checkbox_prefix(&line) {
            // Calculate the new cursor column: original indent + "- [ ] " length
            let old_col = self.cursor_col;
            let old_len = line.graphemes(true).count();
            let new_len = new_line.graphemes(true).count();
            let added = new_len.saturating_sub(old_len);

            self.lines[self.cursor_row] = new_line;
            self.cursor_col = old_col + added;
        }
    }

    /// Toggle the checkbox on the current line between `- [ ]` and `- [x]`.
    /// Does nothing if the line has no checkbox prefix.
    pub fn toggle_checkbox(&mut self) {
        let line = self.current_line().to_string();
        if let Some(new_line) = list_prefix::toggle_checkbox_prefix(&line) {
            self.lines[self.cursor_row] = new_line;
        }
    }

    /// Delete current line and return its content (with trailing newline).
    pub fn delete_line_and_return(&mut self) -> String {
        let line = self.lines[self.cursor_row].clone();
        self.delete_line();
        format!("{}\n", line)
    }

    /// Extract the content of the current line (with trailing newline) without modifying the buffer.
    pub fn extract_line(&self) -> String {
        let line = &self.lines[self.cursor_row];
        format!("{}\n", line)
    }

    /// Compute the cursor position after applying a motion, without mutating self.
    fn position_after_motion(&self, motion: Motion) -> (usize, usize) {
        let mut clone = self.clone();
        clone.history = UndoHistory::default(); // avoid cloning history
        match motion {
            Motion::WordForward => clone.move_word_forward(),
            Motion::WordBackward => clone.move_word_backward(),
            Motion::LineEnd => clone.move_to_line_end(),
            Motion::LineStart => clone.move_to_line_start(),
            Motion::FileEnd => clone.move_to_last_line(),
            Motion::FileStart => clone.move_to_first_line(),
            Motion::Line => {} // handled separately
        }
        (clone.cursor_row, clone.cursor_col)
    }

    /// Delete text covered by a motion. Returns (deleted_text, is_linewise).
    pub fn apply_motion_delete(&mut self, motion: Motion) -> (String, bool) {
        match motion {
            Motion::Line => {
                let text = self.delete_line_and_return();
                (text, true)
            }
            _ => {
                let (end_row, end_col) = self.position_after_motion(motion);
                let (start_row, start_col) = (self.cursor_row, self.cursor_col);

                let ((sr, sc), (er, ec)) = if (start_row, start_col) <= (end_row, end_col) {
                    ((start_row, start_col), (end_row, end_col))
                } else {
                    ((end_row, end_col), (start_row, start_col))
                };

                let text = self.delete_range(sr, sc, er, ec);
                self.cursor_row = sr;
                self.cursor_col = sc;
                (text, false)
            }
        }
    }

    /// Yank (copy) text covered by a motion. Returns (yanked_text, is_linewise).
    pub fn apply_motion_yank(&self, motion: Motion) -> (String, bool) {
        match motion {
            Motion::Line => {
                let text = self.extract_line();
                (text, true)
            }
            _ => {
                let (end_row, end_col) = self.position_after_motion(motion);
                let (start_row, start_col) = (self.cursor_row, self.cursor_col);

                let ((sr, sc), (er, ec)) = if (start_row, start_col) <= (end_row, end_col) {
                    ((start_row, start_col), (end_row, end_col))
                } else {
                    ((end_row, end_col), (start_row, start_col))
                };

                let text = self.extract_range(sr, sc, er, ec);
                (text, false)
            }
        }
    }

    /// Delete a character-wise range and return the deleted text.
    pub fn delete_range(
        &mut self,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> String {
        if start_row == end_row {
            let line = &self.lines[start_row];
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let sc = start_col.min(graphemes.len());
            let ec = end_col.min(graphemes.len());
            let deleted: String = graphemes[sc..ec].iter().copied().collect();
            let remaining: String = graphemes[..sc]
                .iter()
                .chain(graphemes[ec..].iter())
                .copied()
                .collect();
            self.lines[start_row] = remaining;
            deleted
        } else {
            let first_line = &self.lines[start_row];
            let first_graphemes: Vec<&str> = first_line.graphemes(true).collect();
            let sc = start_col.min(first_graphemes.len());

            let last_line = &self.lines[end_row];
            let last_graphemes: Vec<&str> = last_line.graphemes(true).collect();
            let ec = end_col.min(last_graphemes.len());

            // Build deleted text
            let mut deleted = String::new();
            deleted.push_str(&first_graphemes[sc..].iter().copied().collect::<String>());
            deleted.push('\n');
            for row in (start_row + 1)..end_row {
                deleted.push_str(&self.lines[row]);
                deleted.push('\n');
            }
            deleted.push_str(&last_graphemes[..ec].iter().copied().collect::<String>());

            // Merge first and last line portions
            let merged: String = first_graphemes[..sc]
                .iter()
                .chain(last_graphemes[ec..].iter())
                .copied()
                .collect();

            // Remove middle + last lines, replace first
            for _ in (start_row + 1)..=end_row {
                self.lines.remove(start_row + 1);
            }
            self.lines[start_row] = merged;
            deleted
        }
    }

    /// Extract a character-wise range without modifying the buffer.
    pub fn extract_range(
        &self,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> String {
        if start_row == end_row {
            let line = &self.lines[start_row];
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let sc = start_col.min(graphemes.len());
            let ec = end_col.min(graphemes.len());
            graphemes[sc..ec].iter().copied().collect()
        } else {
            let first_graphemes: Vec<&str> = self.lines[start_row].graphemes(true).collect();
            let sc = start_col.min(first_graphemes.len());
            let last_graphemes: Vec<&str> = self.lines[end_row].graphemes(true).collect();
            let ec = end_col.min(last_graphemes.len());

            let mut result = String::new();
            result.push_str(&first_graphemes[sc..].iter().copied().collect::<String>());
            result.push('\n');
            for row in (start_row + 1)..end_row {
                result.push_str(&self.lines[row]);
                result.push('\n');
            }
            result.push_str(&last_graphemes[..ec].iter().copied().collect::<String>());
            result
        }
    }

    /// Paste text after the cursor (character-wise).
    pub fn paste_after_cursor(&mut self, text: &str) {
        let grapheme_count = self.lines[self.cursor_row].graphemes(true).count();
        let insert_pos = (self.cursor_col + 1).min(grapheme_count);
        self.paste_charwise(text, insert_pos);
    }

    /// Paste text before the cursor (character-wise).
    pub fn paste_before_cursor(&mut self, text: &str) {
        let grapheme_count = self.lines[self.cursor_row].graphemes(true).count();
        let insert_pos = self.cursor_col.min(grapheme_count);
        self.paste_charwise(text, insert_pos);
    }

    /// Shared helper for character-wise paste. Inserts `text` at grapheme
    /// position `insert_pos` on the current line, splitting on `\n` so that
    /// multi-line clipboard content creates separate `self.lines` entries.
    fn paste_charwise(&mut self, text: &str, insert_pos: usize) {
        let line = &self.lines[self.cursor_row];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let insert_pos = insert_pos.min(graphemes.len());

        let before: String = graphemes[..insert_pos].iter().copied().collect();
        let after: String = graphemes[insert_pos..].iter().copied().collect();

        if !text.contains('\n') {
            // Single-line: keep existing behaviour
            self.lines[self.cursor_row] = format!("{}{}{}", before, text, after);
            self.cursor_col = insert_pos + text.graphemes(true).count().saturating_sub(1);
            return;
        }

        // Multi-line: split pasted text on '\n'
        let pasted: Vec<&str> = text.split('\n').collect();
        let last_idx = pasted.len() - 1;

        // First segment joins with text before cursor
        self.lines[self.cursor_row] = format!("{}{}", before, pasted[0]);

        // Middle segments become their own lines
        for i in 1..last_idx {
            self.lines
                .insert(self.cursor_row + i, pasted[i].to_string());
        }

        // Last segment joins with text after cursor
        let last_pasted = pasted[last_idx];
        self.lines
            .insert(self.cursor_row + last_idx, format!("{}{}", last_pasted, after));

        self.cursor_row += last_idx;
        self.cursor_col = last_pasted.graphemes(true).count().saturating_sub(1);
    }

    /// Paste line(s) below the current line (line-wise).
    pub fn paste_line_below(&mut self, text: &str) {
        let lines_to_insert: Vec<String> = text
            .lines()
            .map(String::from)
            .collect();
        if lines_to_insert.is_empty() {
            return;
        }
        let insert_at = self.cursor_row + 1;
        for (i, line) in lines_to_insert.iter().enumerate() {
            self.lines.insert(insert_at + i, line.clone());
        }
        self.cursor_row = insert_at;
        self.cursor_col = 0;
    }

    /// Paste line(s) above the current line (line-wise).
    pub fn paste_line_above(&mut self, text: &str) {
        let lines_to_insert: Vec<String> = text
            .lines()
            .map(String::from)
            .collect();
        if lines_to_insert.is_empty() {
            return;
        }
        let insert_at = self.cursor_row;
        for (i, line) in lines_to_insert.iter().enumerate() {
            self.lines.insert(insert_at + i, line.clone());
        }
        self.cursor_row = insert_at;
        self.cursor_col = 0;
    }

    /// Save a snapshot of the current buffer state for undo.
    pub fn save_undo_snapshot(&mut self) {
        let snapshot = BufferSnapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        };
        self.history.undo_stack.push(snapshot);
        if self.history.undo_stack.len() > MAX_UNDO_LEVELS {
            self.history.undo_stack.remove(0);
        }
        self.history.redo_stack.clear();
    }

    /// Undo the last change, returning true if successful.
    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.history.undo_stack.pop() {
            let current = BufferSnapshot {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            };
            self.history.redo_stack.push(current);
            self.lines = snapshot.lines;
            self.cursor_row = snapshot.cursor_row;
            self.cursor_col = snapshot.cursor_col;
            true
        } else {
            false
        }
    }

    /// Redo the last undone change, returning true if successful.
    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.history.redo_stack.pop() {
            let current = BufferSnapshot {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            };
            self.history.undo_stack.push(current);
            self.lines = snapshot.lines;
            self.cursor_row = snapshot.cursor_row;
            self.cursor_col = snapshot.cursor_col;
            true
        } else {
            false
        }
    }

    /// Search forward from (start_row, start_col+1) for `query` (case-insensitive).
    /// Wraps around to the beginning of the buffer.
    /// Returns Some((row, col)) of the match start, or None if not found.
    pub fn find_next(&self, query: &str, start_row: usize, start_col: usize) -> Option<(usize, usize)> {
        if query.is_empty() || self.lines.is_empty() {
            return None;
        }
        let query_lower = query.to_lowercase();
        let total_lines = self.lines.len();

        // Search current line from start_col+1 onwards
        if let Some(col) = self.find_in_line_forward(&self.lines[start_row], start_col + 1, &query_lower) {
            return Some((start_row, col));
        }

        // Search subsequent lines, wrapping around
        for offset in 1..total_lines {
            let row = (start_row + offset) % total_lines;
            if let Some(col) = self.find_in_line_forward(&self.lines[row], 0, &query_lower) {
                return Some((row, col));
            }
        }

        // Search the start line from column 0 up to start_col
        if let Some(col) = self.find_in_line_forward(&self.lines[start_row], 0, &query_lower)
            && col <= start_col
        {
            return Some((start_row, col));
        }

        None
    }

    /// Search backward from (start_row, start_col-1) for `query` (case-insensitive).
    /// Wraps around to the end of the buffer.
    /// Returns Some((row, col)) of the match start, or None if not found.
    pub fn find_prev(&self, query: &str, start_row: usize, start_col: usize) -> Option<(usize, usize)> {
        if query.is_empty() || self.lines.is_empty() {
            return None;
        }
        let query_lower = query.to_lowercase();
        let total_lines = self.lines.len();

        // Search current line backward from start_col-1
        if start_col > 0
            && let Some(col) = self.find_in_line_backward(&self.lines[start_row], start_col - 1, &query_lower)
        {
            return Some((start_row, col));
        }

        // Search preceding lines, wrapping around
        for offset in 1..total_lines {
            let row = (start_row + total_lines - offset) % total_lines;
            let line_len = self.lines[row].graphemes(true).count();
            if let Some(col) = self.find_in_line_backward(&self.lines[row], line_len, &query_lower) {
                return Some((row, col));
            }
        }

        // Search the start line from the end
        let line_len = self.lines[start_row].graphemes(true).count();
        if line_len > start_col
            && let Some(col) = self.find_in_line_backward(&self.lines[start_row], line_len, &query_lower)
            && col > start_col
        {
            return Some((start_row, col));
        }

        None
    }

    /// Find all occurrences of `query` (case-insensitive) in the buffer.
    /// Returns a Vec of (row, col, length_in_graphemes) for each match.
    pub fn find_all(&self, query: &str) -> Vec<(usize, usize, usize)> {
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        let query_grapheme_len = query_lower.graphemes(true).count();
        let mut results = Vec::new();

        for (row, line) in self.lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let mut search_from_byte = 0;

            while let Some(byte_pos) = line_lower[search_from_byte..].find(&query_lower) {
                let abs_byte = search_from_byte + byte_pos;
                // Convert byte offset to grapheme index
                let col = line[..abs_byte].graphemes(true).count();
                results.push((row, col, query_grapheme_len));
                // Advance past this match by at least one grapheme's bytes
                let advance = if col < graphemes.len() {
                    graphemes[col].len()
                } else {
                    1
                };
                search_from_byte = abs_byte + advance;
            }
        }

        results
    }

    /// Find the first occurrence of `query_lower` in `line` starting at grapheme index `from_col`.
    /// Returns the grapheme index of the match start, or None.
    fn find_in_line_forward(&self, line: &str, from_col: usize, query_lower: &str) -> Option<usize> {
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        if from_col >= graphemes.len() {
            return None;
        }

        // Build the substring from from_col and its byte offset
        let byte_offset: usize = graphemes[..from_col].iter().map(|g| g.len()).sum();
        let search_str = &line[byte_offset..];
        let search_lower = search_str.to_lowercase();

        if let Some(byte_match) = search_lower.find(query_lower) {
            // Convert byte match position back to grapheme index
            let matched_bytes = &search_str[..byte_match];
            let grapheme_offset = matched_bytes.graphemes(true).count();
            Some(from_col + grapheme_offset)
        } else {
            None
        }
    }

    /// Find the last occurrence of `query_lower` in `line` at or before grapheme index `before_col`.
    /// Returns the grapheme index of the match start, or None.
    fn find_in_line_backward(&self, line: &str, before_col: usize, query_lower: &str) -> Option<usize> {
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        if graphemes.is_empty() {
            return None;
        }

        let end_col = before_col.min(graphemes.len());
        let byte_end: usize = graphemes[..end_col].iter().map(|g| g.len()).sum();
        let search_str = &line[..byte_end];
        let search_lower = search_str.to_lowercase();

        if let Some(byte_match) = search_lower.rfind(query_lower) {
            let matched_bytes = &search_str[..byte_match];
            let grapheme_offset = matched_bytes.graphemes(true).count();
            Some(grapheme_offset)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_from_string() {
        let buffer = TextBuffer::from_string("Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.to_string(), "Hello\nWorld");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('a');
        buffer.insert_char('b');
        assert_eq!(buffer.to_string(), "ab");
    }

    #[test]
    fn test_backspace() {
        let mut buffer = TextBuffer::from_string("abc");
        buffer.set_cursor(0, 3);
        buffer.backspace();
        assert_eq!(buffer.to_string(), "ab");
    }

    #[test]
    fn test_newline() {
        let mut buffer = TextBuffer::from_string("hello");
        buffer.set_cursor(0, 2);
        buffer.insert_newline();
        assert_eq!(buffer.to_string(), "he\nllo");
    }

    #[test]
    fn test_current_line_content() {
        let buffer = TextBuffer::from_string("first\nsecond");
        assert_eq!(buffer.current_line_content(), "first");
    }

    #[test]
    fn test_insert_line_below_with_prefix() {
        let mut buffer = TextBuffer::from_string("- [ ] task one");
        buffer.insert_line_below_with_prefix("- [ ] ");
        assert_eq!(buffer.to_string(), "- [ ] task one\n- [ ] ");
        assert_eq!(buffer.cursor_position(), (1, 6));
    }

    #[test]
    fn test_insert_newline_with_prefix() {
        let mut buffer = TextBuffer::from_string("- [ ] hello world");
        buffer.set_cursor(0, 12); // after "hello "
        buffer.insert_newline_with_prefix("- [ ] ");
        assert_eq!(buffer.to_string(), "- [ ] hello \n- [ ] world");
        assert_eq!(buffer.cursor_position(), (1, 6));
    }

    #[test]
    fn test_clear_current_line() {
        let mut buffer = TextBuffer::from_string("- [ ] ");
        buffer.set_cursor(0, 6);
        buffer.clear_current_line();
        assert_eq!(buffer.to_string(), "");
        assert_eq!(buffer.cursor_position(), (0, 0));
    }

    #[test]
    fn test_insert_checkbox_on_plain_line() {
        let mut buffer = TextBuffer::from_string("buy milk");
        buffer.set_cursor(0, 3); // cursor on 'm'
        buffer.insert_checkbox();
        assert_eq!(buffer.to_string(), "- [ ] buy milk");
        // cursor shifted by 6 ("- [ ] " length)
        assert_eq!(buffer.cursor_position(), (0, 9));
    }

    #[test]
    fn test_insert_checkbox_already_exists() {
        let mut buffer = TextBuffer::from_string("- [ ] already");
        buffer.set_cursor(0, 8);
        buffer.insert_checkbox();
        // No change
        assert_eq!(buffer.to_string(), "- [ ] already");
        assert_eq!(buffer.cursor_position(), (0, 8));
    }

    #[test]
    fn test_insert_checkbox_indented() {
        let mut buffer = TextBuffer::from_string("    indented text");
        buffer.set_cursor(0, 4);
        buffer.insert_checkbox();
        assert_eq!(buffer.to_string(), "    - [ ] indented text");
        assert_eq!(buffer.cursor_position(), (0, 10));
    }

    #[test]
    fn test_undo_basic() {
        let mut buffer = TextBuffer::from_string("hello");
        buffer.save_undo_snapshot();
        buffer.set_cursor(0, 5);
        buffer.insert_char('!');
        assert_eq!(buffer.to_string(), "hello!");

        assert!(buffer.undo());
        assert_eq!(buffer.to_string(), "hello");
        assert_eq!(buffer.cursor_position(), (0, 0));
    }

    #[test]
    fn test_redo_basic() {
        let mut buffer = TextBuffer::from_string("hello");
        buffer.save_undo_snapshot();
        buffer.set_cursor(0, 5);
        buffer.insert_char('!');
        assert_eq!(buffer.to_string(), "hello!");

        buffer.undo();
        assert_eq!(buffer.to_string(), "hello");

        assert!(buffer.redo());
        assert_eq!(buffer.to_string(), "hello!");
    }

    #[test]
    fn test_undo_limit_50() {
        let mut buffer = TextBuffer::new();
        for i in 0..60 {
            buffer.save_undo_snapshot();
            buffer.insert_char(char::from(b'a' + (i % 26)));
        }
        let mut count = 0;
        while buffer.undo() {
            count += 1;
        }
        assert_eq!(count, 50);
    }

    #[test]
    fn test_undo_clears_redo() {
        let mut buffer = TextBuffer::from_string("hello");
        buffer.save_undo_snapshot();
        buffer.set_cursor(0, 5);
        buffer.insert_char('!');

        buffer.undo();
        assert_eq!(buffer.to_string(), "hello");

        buffer.save_undo_snapshot();
        buffer.set_cursor(0, 5);
        buffer.insert_char('?');
        assert_eq!(buffer.to_string(), "hello?");

        assert!(!buffer.redo());
    }

    #[test]
    fn test_delete_range_same_line() {
        let mut buffer = TextBuffer::from_string("hello world");
        let deleted = buffer.delete_range(0, 0, 0, 5);
        assert_eq!(deleted, "hello");
        assert_eq!(buffer.to_string(), " world");
    }

    #[test]
    fn test_delete_range_multi_line() {
        let mut buffer = TextBuffer::from_string("hello\nworld\nfoo");
        let deleted = buffer.delete_range(0, 3, 1, 3);
        assert_eq!(deleted, "lo\nwor");
        assert_eq!(buffer.to_string(), "helld\nfoo");
    }

    #[test]
    fn test_extract_range() {
        let buffer = TextBuffer::from_string("hello world");
        let text = buffer.extract_range(0, 0, 0, 5);
        assert_eq!(text, "hello");
        assert_eq!(buffer.to_string(), "hello world"); // unchanged
    }

    #[test]
    fn test_apply_motion_delete_word() {
        let mut buffer = TextBuffer::from_string("hello world");
        buffer.set_cursor(0, 0);
        let (text, linewise) = buffer.apply_motion_delete(Motion::WordForward);
        assert_eq!(text, "hello ");
        assert!(!linewise);
        assert_eq!(buffer.to_string(), "world");
    }

    #[test]
    fn test_apply_motion_delete_line() {
        let mut buffer = TextBuffer::from_string("first\nsecond\nthird");
        buffer.set_cursor(1, 0);
        let (text, linewise) = buffer.apply_motion_delete(Motion::Line);
        assert_eq!(text, "second\n");
        assert!(linewise);
        assert_eq!(buffer.to_string(), "first\nthird");
    }

    #[test]
    fn test_apply_motion_yank_line() {
        let buffer = TextBuffer::from_string("first\nsecond");
        let (text, linewise) = buffer.apply_motion_yank(Motion::Line);
        assert_eq!(text, "first\n");
        assert!(linewise);
        assert_eq!(buffer.to_string(), "first\nsecond"); // unchanged
    }

    #[test]
    fn test_paste_after_cursor() {
        let mut buffer = TextBuffer::from_string("helo");
        buffer.set_cursor(0, 1);
        buffer.paste_after_cursor("l");
        assert_eq!(buffer.to_string(), "hello");
    }

    #[test]
    fn test_paste_line_below() {
        let mut buffer = TextBuffer::from_string("first\nthird");
        buffer.set_cursor(0, 0);
        buffer.paste_line_below("second");
        assert_eq!(buffer.to_string(), "first\nsecond\nthird");
        assert_eq!(buffer.cursor_position(), (1, 0));
    }

    #[test]
    fn test_paste_line_above() {
        let mut buffer = TextBuffer::from_string("second\nthird");
        buffer.set_cursor(0, 0);
        buffer.paste_line_above("first");
        assert_eq!(buffer.to_string(), "first\nsecond\nthird");
        assert_eq!(buffer.cursor_position(), (0, 0));
    }

    #[test]
    fn test_toggle_checkbox_check() {
        let mut buffer = TextBuffer::from_string("- [ ] buy milk");
        buffer.set_cursor(0, 8);
        buffer.toggle_checkbox();
        assert_eq!(buffer.to_string(), "- [x] buy milk");
        assert_eq!(buffer.cursor_position(), (0, 8));
    }

    #[test]
    fn test_toggle_checkbox_uncheck() {
        let mut buffer = TextBuffer::from_string("- [x] buy milk");
        buffer.set_cursor(0, 8);
        buffer.toggle_checkbox();
        assert_eq!(buffer.to_string(), "- [ ] buy milk");
        assert_eq!(buffer.cursor_position(), (0, 8));
    }

    #[test]
    fn test_toggle_checkbox_no_checkbox() {
        let mut buffer = TextBuffer::from_string("plain text");
        buffer.set_cursor(0, 3);
        buffer.toggle_checkbox();
        assert_eq!(buffer.to_string(), "plain text");
        assert_eq!(buffer.cursor_position(), (0, 3));
    }

    #[test]
    fn test_find_next_basic() {
        let buffer = TextBuffer::from_string("hello world hello");
        // Starting at (0, 0), find_next should find "world" at col 6
        let result = buffer.find_next("world", 0, 0);
        assert_eq!(result, Some((0, 6)));
    }

    #[test]
    fn test_find_next_skips_current_position() {
        let buffer = TextBuffer::from_string("hello world hello");
        // Cursor at col 6 (start of "world"), find_next should skip it and find "hello" at col 12
        let result = buffer.find_next("hello", 0, 0);
        assert_eq!(result, Some((0, 12)));
    }

    #[test]
    fn test_find_next_wraps() {
        let buffer = TextBuffer::from_string("hello\nworld\nfoo");
        // Start at last line, should wrap to find "hello" at line 0
        let result = buffer.find_next("hello", 2, 0);
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_find_next_case_insensitive() {
        let buffer = TextBuffer::from_string("Hello World HELLO");
        let result = buffer.find_next("hello", 0, 0);
        assert_eq!(result, Some((0, 12)));
        // Also find the first "Hello" when wrapping
        let result2 = buffer.find_next("hello", 0, 12);
        assert_eq!(result2, Some((0, 0)));
    }

    #[test]
    fn test_find_next_no_match() {
        let buffer = TextBuffer::from_string("hello world");
        let result = buffer.find_next("xyz", 0, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_next_empty_query() {
        let buffer = TextBuffer::from_string("hello");
        let result = buffer.find_next("", 0, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_prev_basic() {
        let buffer = TextBuffer::from_string("hello world hello");
        // Cursor at col 12 (second "hello"), find_prev should find "world" at col 6
        let result = buffer.find_prev("world", 0, 12);
        assert_eq!(result, Some((0, 6)));
    }

    #[test]
    fn test_find_prev_wraps() {
        let buffer = TextBuffer::from_string("hello\nworld\nfoo");
        // Start at line 0, should wrap backward to find "foo" at line 2
        let result = buffer.find_prev("foo", 0, 0);
        assert_eq!(result, Some((2, 0)));
    }

    #[test]
    fn test_find_next_multiline() {
        let buffer = TextBuffer::from_string("aaa\nbbb\nccc\naaa");
        // From row 0 col 0, find_next "aaa" should find the one on line 3
        let result = buffer.find_next("aaa", 0, 0);
        assert_eq!(result, Some((3, 0)));
    }

    #[test]
    fn test_find_prev_multiline() {
        let buffer = TextBuffer::from_string("aaa\nbbb\nccc\naaa");
        // From row 3 col 0, find_prev "aaa" should find the one on line 0
        let result = buffer.find_prev("aaa", 3, 0);
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_find_next_unicode() {
        let buffer = TextBuffer::from_string("café résumé café");
        // Search for "résumé" — grapheme positions: c(0) a(1) f(2) é(3) (4) r(5) é(6) s(7) u(8) m(9) é(10) (11) c(12) a(13) f(14) é(15)
        let result = buffer.find_next("résumé", 0, 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_find_next_chinese() {
        let buffer = TextBuffer::from_string("你好世界你好");
        // Search for "世界", should find at grapheme index 2
        let result = buffer.find_next("世界", 0, 0);
        assert_eq!(result, Some((0, 2)));
    }

    #[test]
    fn test_find_all_basic() {
        let buffer = TextBuffer::from_string("hello world hello");
        let matches = buffer.find_all("hello");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 0, 5));
        assert_eq!(matches[1], (0, 12, 5));
    }

    #[test]
    fn test_find_all_case_insensitive() {
        let buffer = TextBuffer::from_string("Hello HELLO hello");
        let matches = buffer.find_all("hello");
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], (0, 0, 5));
        assert_eq!(matches[1], (0, 6, 5));
        assert_eq!(matches[2], (0, 12, 5));
    }

    #[test]
    fn test_find_all_empty_query() {
        let buffer = TextBuffer::from_string("hello");
        let matches = buffer.find_all("");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_all_multiline() {
        let buffer = TextBuffer::from_string("foo bar\nbaz foo\nfoo");
        let matches = buffer.find_all("foo");
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], (0, 0, 3));
        assert_eq!(matches[1], (1, 4, 3));
        assert_eq!(matches[2], (2, 0, 3));
    }

    #[test]
    fn test_paste_after_cursor_multiline() {
        let mut buffer = TextBuffer::from_string("hello world");
        buffer.set_cursor(0, 4); // cursor on 'o'
        buffer.paste_after_cursor("foo\nbar");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.content()[0], "hellofoo");
        assert_eq!(buffer.content()[1], "bar world");
        assert_eq!(buffer.cursor_position(), (1, 2)); // end of "bar"
    }

    #[test]
    fn test_paste_after_cursor_three_lines() {
        let mut buffer = TextBuffer::from_string("AB");
        buffer.set_cursor(0, 0); // cursor on 'A'
        buffer.paste_after_cursor("x\ny\nz");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.content()[0], "Ax");
        assert_eq!(buffer.content()[1], "y");
        assert_eq!(buffer.content()[2], "zB");
        assert_eq!(buffer.cursor_position(), (2, 0)); // on 'z'
    }

    #[test]
    fn test_paste_after_cursor_trailing_newline() {
        let mut buffer = TextBuffer::from_string("AB");
        buffer.set_cursor(0, 0); // cursor on 'A'
        buffer.paste_after_cursor("line1\n");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.content()[0], "Aline1");
        assert_eq!(buffer.content()[1], "B");
        assert_eq!(buffer.cursor_position().0, 1);
    }

    #[test]
    fn test_paste_before_cursor_multiline() {
        let mut buffer = TextBuffer::from_string("hello world");
        buffer.set_cursor(0, 5); // cursor on ' '
        buffer.paste_before_cursor("foo\nbar");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.content()[0], "hellofoo");
        assert_eq!(buffer.content()[1], "bar world");
        assert_eq!(buffer.cursor_position(), (1, 2));
    }

    #[test]
    fn test_paste_after_cursor_single_line_unchanged() {
        // Regression: single-line paste should still work
        let mut buffer = TextBuffer::from_string("helo");
        buffer.set_cursor(0, 1); // cursor on 'e'
        buffer.paste_after_cursor("l");
        assert_eq!(buffer.to_string(), "hello");
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_insert_text_multiline() {
        let mut buffer = TextBuffer::new();
        buffer.insert_text("hello\nworld\nfoo");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.content()[0], "hello");
        assert_eq!(buffer.content()[1], "world");
        assert_eq!(buffer.content()[2], "foo");
        assert_eq!(buffer.cursor_position(), (2, 3));
    }

    #[test]
    fn test_insert_char_newline() {
        let mut buffer = TextBuffer::from_string("hello");
        buffer.set_cursor(0, 3);
        buffer.insert_char('\n');
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.content()[0], "hel");
        assert_eq!(buffer.content()[1], "lo");
        assert_eq!(buffer.cursor_position(), (1, 0));
    }
}

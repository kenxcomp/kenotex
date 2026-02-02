use unicode_segmentation::UnicodeSegmentation;

use super::list_prefix;

#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
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

        while pos < graphemes.len() && graphemes[pos].chars().all(|c| c.is_whitespace()) {
            pos += 1;
        }

        while pos < graphemes.len() && !graphemes[pos].chars().all(|c| c.is_whitespace()) {
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
}

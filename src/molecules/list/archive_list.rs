use crate::types::Note;

#[derive(Debug, Clone, Default)]
pub struct ArchiveList {
    notes: Vec<Note>,
    selected_index: usize,
    search_query: String,
    filtered_indices: Vec<usize>,
}

impl ArchiveList {
    pub fn new(notes: Vec<Note>) -> Self {
        let filtered_indices: Vec<usize> = (0..notes.len()).collect();
        Self {
            notes,
            selected_index: 0,
            search_query: String::new(),
            filtered_indices,
        }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn filtered_notes(&self) -> Vec<&Note> {
        self.filtered_indices
            .iter()
            .filter_map(|&idx| self.notes.get(idx))
            .collect()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_note(&self) -> Option<&Note> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.notes.get(idx))
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.update_filter();
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    pub fn remove_search_char(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filter();
    }

    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.notes.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .notes
                .iter()
                .enumerate()
                .filter(|(_, note)| {
                    note.title.to_lowercase().contains(&query_lower)
                        || note.content.to_lowercase().contains(&query_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
        }

        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_indices.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn remove_selected(&mut self) -> Option<Note> {
        if let Some(&real_idx) = self.filtered_indices.get(self.selected_index) {
            let note = self.notes.remove(real_idx);
            self.update_filter();
            if self.selected_index >= self.filtered_indices.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
            Some(note)
        } else {
            None
        }
    }

    pub fn update_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
        self.update_filter();
    }

    pub fn is_empty(&self) -> bool {
        self.filtered_indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn all_note_ids(&self) -> Vec<String> {
        self.notes.iter().map(|n| n.id.clone()).collect()
    }

    pub fn update_single_note(&mut self, updated: Note) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == updated.id) {
            note.title = updated.title;
            note.content = updated.content;
            note.updated_at = updated.updated_at;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_note(id: &str, title: &str, content: &str) -> Note {
        Note {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_archived: true,
            selected: false,
        }
    }

    #[test]
    fn test_new_empty() {
        let list = ArchiveList::new(vec![]);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_new_with_notes() {
        let notes = vec![make_note("1", "A", "a"), make_note("2", "B", "b")];
        let list = ArchiveList::new(notes);
        assert_eq!(list.len(), 2);
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_selected_note() {
        let notes = vec![make_note("1", "A", "a")];
        let list = ArchiveList::new(notes);
        let selected = list.selected_note().unwrap();
        assert_eq!(selected.id, "1");
    }

    #[test]
    fn test_selected_note_empty() {
        let list = ArchiveList::new(vec![]);
        assert!(list.selected_note().is_none());
    }

    #[test]
    fn test_move_up_down() {
        let notes = vec![
            make_note("1", "A", "a"),
            make_note("2", "B", "b"),
            make_note("3", "C", "c"),
        ];
        let mut list = ArchiveList::new(notes);
        list.move_down();
        assert_eq!(list.selected_index(), 1);
        list.move_down();
        assert_eq!(list.selected_index(), 2);
        list.move_down(); // clamped
        assert_eq!(list.selected_index(), 2);
        list.move_up();
        assert_eq!(list.selected_index(), 1);
        list.move_up();
        assert_eq!(list.selected_index(), 0);
        list.move_up(); // clamped
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_search_filter() {
        let notes = vec![
            make_note("1", "Hello World", "content"),
            make_note("2", "Goodbye", "world stuff"),
            make_note("3", "Test", "nothing"),
        ];
        let mut list = ArchiveList::new(notes);
        list.set_search_query("world".to_string());
        assert_eq!(list.len(), 2);
        list.clear_search();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_add_remove_search_char() {
        let notes = vec![make_note("1", "Hello", "c"), make_note("2", "World", "c")];
        let mut list = ArchiveList::new(notes);
        list.add_search_char('h');
        assert_eq!(list.search_query(), "h");
        assert_eq!(list.len(), 1);
        list.remove_search_char();
        assert_eq!(list.search_query(), "");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_remove_selected() {
        let notes = vec![make_note("1", "A", "a"), make_note("2", "B", "b")];
        let mut list = ArchiveList::new(notes);
        let removed = list.remove_selected().unwrap();
        assert_eq!(removed.id, "1");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_remove_selected_empty() {
        let mut list = ArchiveList::new(vec![]);
        assert!(list.remove_selected().is_none());
    }

    #[test]
    fn test_update_notes() {
        let mut list = ArchiveList::new(vec![make_note("1", "A", "a")]);
        let new_notes = vec![make_note("2", "B", "b"), make_note("3", "C", "c")];
        list.update_notes(new_notes);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_all_note_ids() {
        let notes = vec![make_note("x", "A", "a"), make_note("y", "B", "b")];
        let list = ArchiveList::new(notes);
        assert_eq!(list.all_note_ids(), vec!["x", "y"]);
    }

    #[test]
    fn test_update_single_note() {
        let notes = vec![make_note("1", "Old Title", "old content")];
        let mut list = ArchiveList::new(notes);
        let updated = make_note("1", "New Title", "new content");
        list.update_single_note(updated);
        assert_eq!(list.notes()[0].title, "New Title");
        assert_eq!(list.notes()[0].content, "new content");
    }

    #[test]
    fn test_filtered_notes() {
        let notes = vec![
            make_note("1", "Alpha", "content"),
            make_note("2", "Beta", "content"),
        ];
        let list = ArchiveList::new(notes);
        let filtered = list.filtered_notes();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
    }

    #[test]
    fn test_search_cjk() {
        let notes = vec![
            make_note("1", "你好世界", "content"),
            make_note("2", "Hello", "content"),
        ];
        let mut list = ArchiveList::new(notes);
        list.set_search_query("你好".to_string());
        assert_eq!(list.len(), 1);
        assert_eq!(list.selected_note().unwrap().id, "1");
    }
}

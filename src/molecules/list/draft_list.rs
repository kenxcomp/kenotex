use crate::types::Note;

#[derive(Debug, Clone, Default)]
pub struct DraftList {
    notes: Vec<Note>,
    selected_index: usize,
    search_query: String,
    filtered_indices: Vec<usize>,
}

impl DraftList {
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

    pub fn selected_note_mut(&mut self) -> Option<&mut Note> {
        let idx = self.filtered_indices.get(self.selected_index).copied()?;
        self.notes.get_mut(idx)
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

    pub fn add_note(&mut self, note: Note) {
        self.notes.insert(0, note);
        self.update_filter();
        self.selected_index = 0;
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

    pub fn update_note(&mut self, updated: &Note) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == updated.id) {
            note.title = updated.title.clone();
            note.content = updated.content.clone();
            note.updated_at = updated.updated_at;
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(&real_idx) = self.filtered_indices.get(self.selected_index)
            && let Some(note) = self.notes.get_mut(real_idx) {
                note.selected = !note.selected;
            }
    }

    pub fn get_selected_notes(&self) -> Vec<&Note> {
        self.notes.iter().filter(|n| n.selected).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.filtered_indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn total_count(&self) -> usize {
        self.notes.len()
    }

    pub fn all_note_ids(&self) -> Vec<String> {
        self.notes.iter().map(|n| n.id.clone()).collect()
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
            is_archived: false,
            selected: false,
        }
    }

    #[test]
    fn test_search_filter() {
        let notes = vec![
            make_note("1", "Hello World", "content"),
            make_note("2", "Goodbye", "world content"),
            make_note("3", "Test", "nothing"),
        ];

        let mut list = DraftList::new(notes);
        assert_eq!(list.len(), 3);

        list.set_search_query("world".to_string());
        assert_eq!(list.len(), 2);

        list.clear_search();
        assert_eq!(list.len(), 3);
    }
}

use crate::types::Theme;

pub struct ThemeManager {
    themes: Vec<Theme>,
    current_index: usize,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: Theme::all_themes(),
            current_index: 0,
        }
    }

    pub fn with_theme(theme_name: &str) -> Self {
        let themes = Theme::all_themes();
        let current_index = themes
            .iter()
            .position(|t| t.name.to_lowercase().replace(' ', "_") == theme_name.to_lowercase())
            .unwrap_or(0);

        Self {
            themes,
            current_index,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.themes[self.current_index]
    }

    pub fn cycle_next(&mut self) -> &Theme {
        self.current_index = (self.current_index + 1) % self.themes.len();
        self.current()
    }

    pub fn cycle_prev(&mut self) -> &Theme {
        self.current_index = if self.current_index == 0 {
            self.themes.len() - 1
        } else {
            self.current_index - 1
        };
        self.current()
    }

    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(idx) = self
            .themes
            .iter()
            .position(|t| t.name.to_lowercase().replace(' ', "_") == name.to_lowercase())
        {
            self.current_index = idx;
            true
        } else {
            false
        }
    }

    pub fn theme_names(&self) -> Vec<&str> {
        self.themes.iter().map(|t| t.name.as_str()).collect()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycling() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current().name, "Tokyo Night");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Gruvbox");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Nord");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Catppuccin Mocha");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Catppuccin Macchiato");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Catppuccin Frappe");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Catppuccin Latte");

        manager.cycle_next();
        assert_eq!(manager.current().name, "Tokyo Night");
    }

    #[test]
    fn test_set_theme() {
        let mut manager = ThemeManager::new();
        assert!(manager.set_theme("gruvbox"));
        assert_eq!(manager.current().name, "Gruvbox");
    }
}

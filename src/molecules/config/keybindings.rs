use crate::types::KeyboardConfig;

pub struct Keybindings {
    pub move_up: String,
    pub move_down: String,
    pub layout: String,
}

impl Keybindings {
    pub fn from_config(config: &KeyboardConfig) -> Self {
        Self {
            move_up: config.move_up.clone(),
            move_down: config.move_down.clone(),
            layout: config.layout.clone(),
        }
    }

    pub fn qwerty() -> Self {
        Self {
            move_up: "k".to_string(),
            move_down: "j".to_string(),
            layout: "qwerty".to_string(),
        }
    }

    pub fn colemak() -> Self {
        Self {
            move_up: "u".to_string(),
            move_down: "e".to_string(),
            layout: "colemak".to_string(),
        }
    }

    pub fn is_up_key(&self, key: &str) -> bool {
        key == self.move_up
    }

    pub fn is_down_key(&self, key: &str) -> bool {
        key == self.move_down
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::qwerty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KeyboardConfig;

    #[test]
    fn test_qwerty_defaults() {
        let kb = Keybindings::qwerty();
        assert_eq!(kb.move_up, "k");
        assert_eq!(kb.move_down, "j");
        assert_eq!(kb.layout, "qwerty");
    }

    #[test]
    fn test_colemak_defaults() {
        let kb = Keybindings::colemak();
        assert_eq!(kb.move_up, "u");
        assert_eq!(kb.move_down, "e");
        assert_eq!(kb.layout, "colemak");
    }

    #[test]
    fn test_default_is_qwerty() {
        let kb = Keybindings::default();
        assert_eq!(kb.layout, "qwerty");
        assert_eq!(kb.move_up, "k");
        assert_eq!(kb.move_down, "j");
    }

    #[test]
    fn test_from_config_qwerty() {
        let config = KeyboardConfig::default();
        let kb = Keybindings::from_config(&config);
        assert_eq!(kb.move_up, "k");
        assert_eq!(kb.move_down, "j");
        assert_eq!(kb.layout, "qwerty");
    }

    #[test]
    fn test_from_config_colemak() {
        let config = KeyboardConfig::colemak();
        let kb = Keybindings::from_config(&config);
        assert_eq!(kb.move_up, "u");
        assert_eq!(kb.move_down, "e");
        assert_eq!(kb.layout, "colemak");
    }

    #[test]
    fn test_is_up_key() {
        let kb = Keybindings::qwerty();
        assert!(kb.is_up_key("k"));
        assert!(!kb.is_up_key("j"));
        assert!(!kb.is_up_key("u"));
    }

    #[test]
    fn test_is_down_key() {
        let kb = Keybindings::qwerty();
        assert!(kb.is_down_key("j"));
        assert!(!kb.is_down_key("k"));
    }

    #[test]
    fn test_colemak_is_up_key() {
        let kb = Keybindings::colemak();
        assert!(kb.is_up_key("u"));
        assert!(!kb.is_up_key("k"));
    }

    #[test]
    fn test_colemak_is_down_key() {
        let kb = Keybindings::colemak();
        assert!(kb.is_down_key("e"));
        assert!(!kb.is_down_key("j"));
    }
}

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

use crate::types::KeyboardConfig;

pub struct Keybindings {
    pub direction_up: String,
    pub direction_down: String,
    pub layout: String,
}

impl Keybindings {
    pub fn from_config(config: &KeyboardConfig) -> Self {
        Self {
            direction_up: config.direction_up.clone(),
            direction_down: config.direction_down.clone(),
            layout: config.layout.clone(),
        }
    }

    pub fn qwerty() -> Self {
        Self {
            direction_up: "k".to_string(),
            direction_down: "j".to_string(),
            layout: "qwerty".to_string(),
        }
    }

    pub fn colemak() -> Self {
        Self {
            direction_up: "u".to_string(),
            direction_down: "e".to_string(),
            layout: "colemak".to_string(),
        }
    }

    pub fn is_up_key(&self, key: &str) -> bool {
        key == self.direction_up
    }

    pub fn is_down_key(&self, key: &str) -> bool {
        key == self.direction_down
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::qwerty()
    }
}

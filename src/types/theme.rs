use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg: String,
    pub fg: String,
    pub cursor: String,
    pub selection: String,
    pub border: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub panel: String,
}

impl Theme {
    pub fn bg_color(&self) -> Color {
        Self::parse_hex(&self.bg)
    }

    pub fn fg_color(&self) -> Color {
        Self::parse_hex(&self.fg)
    }

    pub fn cursor_color(&self) -> Color {
        Self::parse_hex(&self.cursor)
    }

    pub fn selection_color(&self) -> Color {
        Self::parse_hex(&self.selection)
    }

    pub fn border_color(&self) -> Color {
        Self::parse_hex(&self.border)
    }

    pub fn accent_color(&self) -> Color {
        Self::parse_hex(&self.accent)
    }

    pub fn success_color(&self) -> Color {
        Self::parse_hex(&self.success)
    }

    pub fn warning_color(&self) -> Color {
        Self::parse_hex(&self.warning)
    }

    pub fn error_color(&self) -> Color {
        Self::parse_hex(&self.error)
    }

    pub fn panel_color(&self) -> Color {
        Self::parse_hex(&self.panel)
    }

    fn parse_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        } else {
            Color::Reset
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            bg: "#1a1b26".to_string(),
            fg: "#a9b1d6".to_string(),
            cursor: "#c0caf5".to_string(),
            selection: "#283457".to_string(),
            border: "#414868".to_string(),
            accent: "#7aa2f7".to_string(),
            success: "#9ece6a".to_string(),
            warning: "#e0af68".to_string(),
            error: "#f7768e".to_string(),
            panel: "#16161e".to_string(),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox".to_string(),
            bg: "#282828".to_string(),
            fg: "#ebdbb2".to_string(),
            cursor: "#ebdbb2".to_string(),
            selection: "#504945".to_string(),
            border: "#665c54".to_string(),
            accent: "#d79921".to_string(),
            success: "#98971a".to_string(),
            warning: "#d65d0e".to_string(),
            error: "#cc241d".to_string(),
            panel: "#1d2021".to_string(),
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            bg: "#2e3440".to_string(),
            fg: "#d8dee9".to_string(),
            cursor: "#d8dee9".to_string(),
            selection: "#434c5e".to_string(),
            border: "#4c566a".to_string(),
            accent: "#88c0d0".to_string(),
            success: "#a3be8c".to_string(),
            warning: "#ebcb8b".to_string(),
            error: "#bf616a".to_string(),
            panel: "#242933".to_string(),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            bg: "#1e1e2e".to_string(),
            fg: "#cdd6f4".to_string(),
            cursor: "#b4befe".to_string(),
            selection: "#313244".to_string(),
            border: "#45475a".to_string(),
            accent: "#89b4fa".to_string(),
            success: "#a6e3a1".to_string(),
            warning: "#f9e2af".to_string(),
            error: "#f38ba8".to_string(),
            panel: "#181825".to_string(),
        }
    }

    pub fn catppuccin_macchiato() -> Self {
        Self {
            name: "Catppuccin Macchiato".to_string(),
            bg: "#24273a".to_string(),
            fg: "#cad3f5".to_string(),
            cursor: "#b7bdf8".to_string(),
            selection: "#363a4f".to_string(),
            border: "#494d64".to_string(),
            accent: "#8aadf4".to_string(),
            success: "#a6da95".to_string(),
            warning: "#eed49f".to_string(),
            error: "#ed8796".to_string(),
            panel: "#1e2030".to_string(),
        }
    }

    pub fn catppuccin_frappe() -> Self {
        Self {
            name: "Catppuccin Frappe".to_string(),
            bg: "#303446".to_string(),
            fg: "#c6d0f5".to_string(),
            cursor: "#babbf1".to_string(),
            selection: "#414559".to_string(),
            border: "#51576d".to_string(),
            accent: "#8caaee".to_string(),
            success: "#a6d189".to_string(),
            warning: "#e5c890".to_string(),
            error: "#e78284".to_string(),
            panel: "#292c3c".to_string(),
        }
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            name: "Catppuccin Latte".to_string(),
            bg: "#eff1f5".to_string(),
            fg: "#4c4f69".to_string(),
            cursor: "#7287fd".to_string(),
            selection: "#ccd0da".to_string(),
            border: "#bcc0cc".to_string(),
            accent: "#1e66f5".to_string(),
            success: "#40a02b".to_string(),
            warning: "#df8e1d".to_string(),
            error: "#d20f39".to_string(),
            panel: "#e6e9ef".to_string(),
        }
    }

    pub fn all_themes() -> Vec<Theme> {
        vec![
            Self::tokyo_night(),
            Self::gruvbox(),
            Self::nord(),
            Self::catppuccin_mocha(),
            Self::catppuccin_macchiato(),
            Self::catppuccin_frappe(),
            Self::catppuccin_latte(),
        ]
    }
}

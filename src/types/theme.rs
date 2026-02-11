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
    pub comment: String,
    pub keyword: String,
    pub string: String,
    pub type_name: String,
    pub function: String,
    pub constant: String,
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

    pub fn comment_color(&self) -> Color {
        Self::parse_hex(&self.comment)
    }

    pub fn keyword_color(&self) -> Color {
        Self::parse_hex(&self.keyword)
    }

    pub fn string_color(&self) -> Color {
        Self::parse_hex(&self.string)
    }

    pub fn type_name_color(&self) -> Color {
        Self::parse_hex(&self.type_name)
    }

    pub fn function_color(&self) -> Color {
        Self::parse_hex(&self.function)
    }

    pub fn constant_color(&self) -> Color {
        Self::parse_hex(&self.constant)
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
            comment: "#565f89".to_string(),
            keyword: "#bb9af7".to_string(),
            string: "#9ece6a".to_string(),
            type_name: "#2ac3de".to_string(),
            function: "#7aa2f7".to_string(),
            constant: "#ff9e64".to_string(),
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
            comment: "#928374".to_string(),
            keyword: "#fb4934".to_string(),
            string: "#b8bb26".to_string(),
            type_name: "#83a598".to_string(),
            function: "#fabd2f".to_string(),
            constant: "#d3869b".to_string(),
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
            comment: "#616e88".to_string(),
            keyword: "#81a1c1".to_string(),
            string: "#a3be8c".to_string(),
            type_name: "#88c0d0".to_string(),
            function: "#8fbcbb".to_string(),
            constant: "#b48ead".to_string(),
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
            comment: "#6c7086".to_string(),
            keyword: "#cba6f7".to_string(),
            string: "#a6e3a1".to_string(),
            type_name: "#94e2d5".to_string(),
            function: "#89b4fa".to_string(),
            constant: "#fab387".to_string(),
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
            comment: "#6e738d".to_string(),
            keyword: "#c6a0f6".to_string(),
            string: "#a6da95".to_string(),
            type_name: "#8bd5ca".to_string(),
            function: "#8aadf4".to_string(),
            constant: "#f5a97f".to_string(),
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
            comment: "#737994".to_string(),
            keyword: "#ca9ee6".to_string(),
            string: "#a6d189".to_string(),
            type_name: "#81c8be".to_string(),
            function: "#8caaee".to_string(),
            constant: "#ef9f76".to_string(),
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
            comment: "#9ca0b0".to_string(),
            keyword: "#8839ef".to_string(),
            string: "#40a02b".to_string(),
            type_name: "#179299".to_string(),
            function: "#1e66f5".to_string(),
            constant: "#fe640b".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_all_themes_have_valid_colors() {
        for theme in Theme::all_themes() {
            // Test all 16 fields parse to valid Color (not Color::Reset)
            assert!(
                !matches!(theme.bg_color(), Color::Reset),
                "{}: bg",
                theme.name
            );
            assert!(
                !matches!(theme.fg_color(), Color::Reset),
                "{}: fg",
                theme.name
            );
            assert!(
                !matches!(theme.cursor_color(), Color::Reset),
                "{}: cursor",
                theme.name
            );
            assert!(
                !matches!(theme.selection_color(), Color::Reset),
                "{}: selection",
                theme.name
            );
            assert!(
                !matches!(theme.border_color(), Color::Reset),
                "{}: border",
                theme.name
            );
            assert!(
                !matches!(theme.accent_color(), Color::Reset),
                "{}: accent",
                theme.name
            );
            assert!(
                !matches!(theme.success_color(), Color::Reset),
                "{}: success",
                theme.name
            );
            assert!(
                !matches!(theme.warning_color(), Color::Reset),
                "{}: warning",
                theme.name
            );
            assert!(
                !matches!(theme.error_color(), Color::Reset),
                "{}: error",
                theme.name
            );
            assert!(
                !matches!(theme.panel_color(), Color::Reset),
                "{}: panel",
                theme.name
            );
            assert!(
                !matches!(theme.comment_color(), Color::Reset),
                "{}: comment",
                theme.name
            );
            assert!(
                !matches!(theme.keyword_color(), Color::Reset),
                "{}: keyword",
                theme.name
            );
            assert!(
                !matches!(theme.string_color(), Color::Reset),
                "{}: string",
                theme.name
            );
            assert!(
                !matches!(theme.type_name_color(), Color::Reset),
                "{}: type_name",
                theme.name
            );
            assert!(
                !matches!(theme.function_color(), Color::Reset),
                "{}: function",
                theme.name
            );
            assert!(
                !matches!(theme.constant_color(), Color::Reset),
                "{}: constant",
                theme.name
            );
        }
    }

    #[test]
    fn test_all_themes_count() {
        let themes = Theme::all_themes();
        assert_eq!(themes.len(), 7);
    }

    #[test]
    fn test_tokyo_night_syntax_colors() {
        let t = Theme::tokyo_night();
        assert_eq!(t.comment, "#565f89");
        assert_eq!(t.keyword, "#bb9af7");
        assert_eq!(t.string, "#9ece6a");
        assert_eq!(t.type_name, "#2ac3de");
        assert_eq!(t.function, "#7aa2f7");
        assert_eq!(t.constant, "#ff9e64");
    }

    #[test]
    fn test_gruvbox_syntax_colors() {
        let t = Theme::gruvbox();
        assert_eq!(t.comment, "#928374");
        assert_eq!(t.keyword, "#fb4934");
        assert_eq!(t.string, "#b8bb26");
        assert_eq!(t.type_name, "#83a598");
        assert_eq!(t.function, "#fabd2f");
        assert_eq!(t.constant, "#d3869b");
    }

    #[test]
    fn test_nord_syntax_colors() {
        let t = Theme::nord();
        assert_eq!(t.comment, "#616e88");
        assert_eq!(t.keyword, "#81a1c1");
        assert_eq!(t.string, "#a3be8c");
        assert_eq!(t.type_name, "#88c0d0");
        assert_eq!(t.function, "#8fbcbb");
        assert_eq!(t.constant, "#b48ead");
    }

    #[test]
    fn test_catppuccin_mocha_syntax_colors() {
        let t = Theme::catppuccin_mocha();
        assert_eq!(t.comment, "#6c7086");
        assert_eq!(t.keyword, "#cba6f7");
        assert_eq!(t.string, "#a6e3a1");
        assert_eq!(t.type_name, "#94e2d5");
        assert_eq!(t.function, "#89b4fa");
        assert_eq!(t.constant, "#fab387");
    }

    #[test]
    fn test_catppuccin_macchiato_syntax_colors() {
        let t = Theme::catppuccin_macchiato();
        assert_eq!(t.comment, "#6e738d");
        assert_eq!(t.keyword, "#c6a0f6");
        assert_eq!(t.string, "#a6da95");
        assert_eq!(t.type_name, "#8bd5ca");
        assert_eq!(t.function, "#8aadf4");
        assert_eq!(t.constant, "#f5a97f");
    }

    #[test]
    fn test_catppuccin_frappe_syntax_colors() {
        let t = Theme::catppuccin_frappe();
        assert_eq!(t.comment, "#737994");
        assert_eq!(t.keyword, "#ca9ee6");
        assert_eq!(t.string, "#a6d189");
        assert_eq!(t.type_name, "#81c8be");
        assert_eq!(t.function, "#8caaee");
        assert_eq!(t.constant, "#ef9f76");
    }

    #[test]
    fn test_catppuccin_latte_syntax_colors() {
        let t = Theme::catppuccin_latte();
        assert_eq!(t.comment, "#9ca0b0");
        assert_eq!(t.keyword, "#8839ef");
        assert_eq!(t.string, "#40a02b");
        assert_eq!(t.type_name, "#179299");
        assert_eq!(t.function, "#1e66f5");
        assert_eq!(t.constant, "#fe640b");
    }

    #[test]
    fn test_tokyo_night_differs_from_catppuccin_mocha() {
        let tn = Theme::tokyo_night();
        let cm = Theme::catppuccin_mocha();
        assert_ne!(tn.comment, cm.comment);
        assert_ne!(tn.keyword, cm.keyword);
        assert_ne!(tn.string, cm.string);
        assert_ne!(tn.type_name, cm.type_name);
        assert_ne!(tn.function, cm.function);
        assert_ne!(tn.constant, cm.constant);
    }

    #[test]
    fn test_parse_hex_valid_rgb() {
        let t = Theme::tokyo_night();
        // "#1a1b26" => RGB(0x1a, 0x1b, 0x26)
        match t.bg_color() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0x1a);
                assert_eq!(g, 0x1b);
                assert_eq!(b, 0x26);
            }
            other => panic!("Expected Color::Rgb, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_hex_comment_color_rgb() {
        let t = Theme::tokyo_night();
        // "#565f89" => RGB(0x56, 0x5f, 0x89)
        match t.comment_color() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0x56);
                assert_eq!(g, 0x5f);
                assert_eq!(b, 0x89);
            }
            other => panic!("Expected Color::Rgb, got {:?}", other),
        }
    }

    #[test]
    fn test_all_syntax_fields_are_hex_format() {
        for theme in Theme::all_themes() {
            for (field_name, value) in [
                ("comment", &theme.comment),
                ("keyword", &theme.keyword),
                ("string", &theme.string),
                ("type_name", &theme.type_name),
                ("function", &theme.function),
                ("constant", &theme.constant),
            ] {
                assert!(
                    value.starts_with('#'),
                    "{}: {} should start with #, got {}",
                    theme.name,
                    field_name,
                    value
                );
                assert_eq!(
                    value.len(),
                    7,
                    "{}: {} should be 7 chars (#RRGGBB), got {}",
                    theme.name,
                    field_name,
                    value
                );
            }
        }
    }

    #[test]
    fn test_theme_names_are_unique() {
        let themes = Theme::all_themes();
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(name, other, "Duplicate theme name: {}", name);
                }
            }
        }
    }

    #[test]
    fn test_gruvbox_differs_from_nord() {
        let g = Theme::gruvbox();
        let n = Theme::nord();
        assert_ne!(g.comment, n.comment);
        assert_ne!(g.keyword, n.keyword);
        assert_ne!(g.type_name, n.type_name);
        assert_ne!(g.function, n.function);
        assert_ne!(g.constant, n.constant);
    }

    #[test]
    fn test_catppuccin_variants_differ() {
        let mocha = Theme::catppuccin_mocha();
        let macchiato = Theme::catppuccin_macchiato();
        let frappe = Theme::catppuccin_frappe();
        let latte = Theme::catppuccin_latte();

        // Each variant should have distinct syntax colors
        assert_ne!(mocha.comment, macchiato.comment);
        assert_ne!(mocha.comment, frappe.comment);
        assert_ne!(mocha.comment, latte.comment);
        assert_ne!(macchiato.keyword, frappe.keyword);
        assert_ne!(macchiato.keyword, latte.keyword);
        assert_ne!(frappe.constant, latte.constant);
    }
}

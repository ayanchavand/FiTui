#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::models::TransactionType;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ThemeConfig {
    pub accent: String,
    pub accent_soft: String,
    pub credit: String,
    pub debit: String,
    pub muted: String,
    pub subtle: String,
    pub background: String,
    pub surface: String,
    pub row_alt: String,
    pub foreground: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub accent: Color,
    pub accent_soft: Color,

    pub credit: Color,
    pub debit: Color,

    pub muted: Color,
    pub subtle: Color,

    pub background: Color,
    pub surface: Color,
    pub row_alt: Color,

    pub foreground: Color,
}

pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::Gray),
        "dark_gray" => Some(Color::DarkGray),
        "light_red" => Some(Color::LightRed),
        "light_green" => Some(Color::LightGreen),
        "light_yellow" => Some(Color::LightYellow),
        "light_blue" => Some(Color::LightBlue),
        "light_magenta" => Some(Color::LightMagenta),
        "light_cyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ if s.starts_with('#') => {
            let hex = &s[1..];
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgb(r, g, b))
            } else if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Color::Rgb(r * 17, g * 17, b * 17))
            } else {
                None
            }
        }
        _ if s.starts_with("rgb(") && s.ends_with(')') => {
            let parts: Vec<&str> = s[4..s.len() - 1].split(',').collect();
            if parts.len() == 3 {
                let r = parts[0].trim().parse::<u8>().ok()?;
                let g = parts[1].trim().parse::<u8>().ok()?;
                let b = parts[2].trim().parse::<u8>().ok()?;
                Some(Color::Rgb(r, g, b))
            } else {
                None
            }
        }
        _ => None,
    }
}

impl Theme {
    pub fn default() -> Self {
        Self {
            accent: Color::Rgb(100, 181, 246),
            accent_soft: Color::Rgb(80, 140, 200),

            credit: Color::Rgb(102, 187, 106),
            debit: Color::Rgb(239, 83, 80),

            muted: Color::Rgb(160, 160, 170),
            subtle: Color::Rgb(90, 90, 110),

            background: Color::Rgb(24, 24, 36),
            surface: Color::Rgb(34, 34, 52),
            row_alt: Color::Rgb(29, 29, 44), // midpoint between background and surface

            foreground: Color::Rgb(220, 225, 245),
        }
    }

    pub fn from_config(config: &ThemeConfig) -> Result<Self, String> {
        let parse = |name: &str, val: &str| {
            parse_color(val).ok_or_else(|| format!("Invalid color for '{}': '{}'", name, val))
        };

        Ok(Self {
            accent: parse("accent", &config.accent)?,
            accent_soft: parse("accent_soft", &config.accent_soft)?,
            credit: parse("credit", &config.credit)?,
            debit: parse("debit", &config.debit)?,
            muted: parse("muted", &config.muted)?,
            subtle: parse("subtle", &config.subtle)?,
            background: parse("background", &config.background)?,
            surface: parse("surface", &config.surface)?,
            row_alt: parse("row_alt", &config.row_alt)?,
            foreground: parse("foreground", &config.foreground)?,
        })
    }

    pub fn get_preconfigured(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default()),
            "dracula" => Some(Self {
                accent: Color::Rgb(189, 147, 249),      // purple
                accent_soft: Color::Rgb(98, 114, 164),  // comment/gray-blue
                credit: Color::Rgb(80, 250, 123),       // green
                debit: Color::Rgb(255, 85, 85),         // red
                muted: Color::Rgb(98, 114, 164),        // comment
                subtle: Color::Rgb(68, 71, 90),         // selection/darker gray
                background: Color::Rgb(40, 42, 54),     // bg
                surface: Color::Rgb(52, 55, 70),        // current line/surface
                row_alt: Color::Rgb(45, 47, 59),        // midpoint
                foreground: Color::Rgb(248, 248, 242),  // fg
            }),
            "nord" => Some(Self {
                accent: Color::Rgb(136, 192, 208),      // frost blue (nord8)
                accent_soft: Color::Rgb(129, 161, 193),  // medium frost blue (nord9)
                credit: Color::Rgb(163, 190, 140),      // green (nord14)
                debit: Color::Rgb(191, 97, 106),        // red (nord11)
                muted: Color::Rgb(76, 86, 106),         // polar night (nord3)
                subtle: Color::Rgb(59, 66, 82),         // polar night (nord1)
                background: Color::Rgb(46, 52, 64),     // polar night (nord0)
                surface: Color::Rgb(67, 76, 94),        // polar night (nord2)
                row_alt: Color::Rgb(53, 60, 74),        // midpoint
                foreground: Color::Rgb(216, 222, 233),  // snow storm (nord4)
            }),
            "gruvbox" | "gruvbox_dark" | "gruvbox-dark" => Some(Self {
                accent: Color::Rgb(250, 189, 47),       // yellow
                accent_soft: Color::Rgb(215, 153, 33),   // darker yellow
                credit: Color::Rgb(184, 187, 38),       // green
                debit: Color::Rgb(251, 73, 52),         // red
                muted: Color::Rgb(146, 131, 116),       // gray
                subtle: Color::Rgb(80, 73, 69),         // dark gray
                background: Color::Rgb(40, 40, 40),      // bg0
                surface: Color::Rgb(60, 56, 54),        // bg1
                row_alt: Color::Rgb(50, 48, 47),        // midpoint
                foreground: Color::Rgb(235, 219, 178),  // fg0
            }),
            _ => None,
        }
    }

        pub fn transaction_color(&self, tx_type: TransactionType) -> Color {
            match tx_type {
                TransactionType::Credit => self.credit,
                TransactionType::Debit => self.debit,
            }
        }

        pub fn danger(&self) -> Style {
            Style::default()
                .fg(self.debit)
                .add_modifier(Modifier::BOLD)
        }

        pub fn success(&self) -> Style {
            Style::default()
                .fg(self.credit)
                .add_modifier(Modifier::BOLD)
        }

        pub fn muted_text(&self) -> Style {
            Style::default().fg(self.muted)
        }

        pub fn title(&self) -> Style {
            Style::default()
                .fg(self.accent)
                .add_modifier(Modifier::BOLD)
        }

        pub fn highlight_style(&self) -> Style {
            Style::default()
                .bg(self.surface)
                .fg(self.foreground)
                .add_modifier(Modifier::BOLD)
        }

        pub fn cursor_style(&self) -> Style {
            Style::default()
                .fg(self.accent)
        }

        pub fn block<'a>(&self, title: &'a str) -> Block<'a> {
            Block::default()
                .title(Span::styled(title, self.title()))
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.accent_soft))
        }

        pub fn panel<'a>(&self) -> Block<'a> {
            Block::default()
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.subtle))
                .style(Style::default().bg(self.background))
        }

        pub fn popup<'a>(&self, title: &'a str) -> Block<'a> {
            Block::default()
                .title(Span::styled(title, self.title()))
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.accent))
                .style(Style::default().bg(self.surface))
        }

        pub fn separator_span(&self) -> Span<'static> {
            Span::styled(" │ ", Style::default().fg(self.subtle))
        }

        pub fn cursor_indicator(&self) -> Span<'static> {
            Span::styled("►", Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
        }

        pub fn bracket_open(&self) -> Span<'static> {
            Span::styled("[", self.muted_text())
        }

        pub fn bracket_close(&self) -> Span<'static> {
            Span::styled("] ", self.muted_text())
        }

        pub fn dimmed_span<'a>(&self, text: &'a str) -> Span<'a> {
            Span::styled(text, Style::default().fg(self.muted))
        }

        pub fn highlight_span<'a>(&self, text: &'a str) -> Span<'a> {
            Span::styled(text, Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
        }
    }

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("  BLUE  "), Some(Color::Blue));
        assert_eq!(parse_color("#ffffff"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(parse_color("#123"), Some(Color::Rgb(17, 34, 51)));
        assert_eq!(parse_color("rgb(10, 20, 30)"), Some(Color::Rgb(10, 20, 30)));
        assert_eq!(parse_color("rgb( 100 , 200 , 255 )"), Some(Color::Rgb(100, 200, 255)));
        assert_eq!(parse_color("invalid"), None);
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("#1234567"), None);
    }

    #[test]
    fn test_theme_from_config() {
        let config = ThemeConfig {
            accent: "red".to_string(),
            accent_soft: "#123".to_string(),
            credit: "green".to_string(),
            debit: "rgb(255,0,0)".to_string(),
            muted: "gray".to_string(),
            subtle: "dark_gray".to_string(),
            background: "black".to_string(),
            surface: "#000000".to_string(),
            row_alt: "#111111".to_string(),
            foreground: "white".to_string(),
        };

        let theme = Theme::from_config(&config).unwrap();
        assert_eq!(theme.accent, Color::Red);
        assert_eq!(theme.accent_soft, Color::Rgb(17, 34, 51));
        assert_eq!(theme.credit, Color::Green);
        assert_eq!(theme.debit, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn test_preconfigured_themes() {
        assert!(Theme::get_preconfigured("dracula").is_some());
        assert!(Theme::get_preconfigured("nord").is_some());
        assert!(Theme::get_preconfigured("gruvbox").is_some());
        assert!(Theme::get_preconfigured("default").is_some());
        assert!(Theme::get_preconfigured("invalid").is_none());
    }
}
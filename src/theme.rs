use ratatui::style::{Color, Style};

/// Nord-inspired terminal color theme
pub struct Theme;

impl Theme {
    // ---- JSON type colors ----
    pub const STRING: Color = Color::Rgb(163, 190, 140);   // soft green
    pub const NUMBER: Color = Color::Rgb(180, 142, 173);   // muted purple
    pub const BOOLEAN: Color = Color::Rgb(208, 135, 112);  // warm orange
    pub const NULL: Color = Color::Rgb(76, 86, 106);       // dim gray
    pub const KEY: Color = Color::Rgb(129, 161, 193);      // steel blue

    // ---- UI colors ----
    pub const BG_SELECTED: Color = Color::Rgb(67, 76, 94);
    pub const BG_STATUS: Color = Color::Rgb(59, 66, 82);
    pub const FG_MUTED: Color = Color::Rgb(129, 161, 193);
    pub const BORDER: Color = Color::Rgb(76, 86, 106);
    pub const ACCENT: Color = Color::Rgb(136, 192, 208);

    // ---- Style helpers ----
    pub fn selected() -> Style {
        Style::default().bg(Self::BG_SELECTED)
    }

    pub fn key() -> Style {
        Style::default().fg(Self::KEY)
    }

    pub fn string() -> Style {
        Style::default().fg(Self::STRING)
    }

    pub fn number() -> Style {
        Style::default().fg(Self::NUMBER)
    }

    pub fn boolean() -> Style {
        Style::default().fg(Self::BOOLEAN)
    }

    pub fn null() -> Style {
        Style::default().fg(Self::NULL)
    }

    pub fn accent() -> Style {
        Style::default().fg(Self::ACCENT)
    }

    pub fn muted() -> Style {
        Style::default().fg(Self::FG_MUTED)
    }

    pub fn status_bar() -> Style {
        Style::default().fg(Self::ACCENT).bg(Self::BG_STATUS)
    }
}

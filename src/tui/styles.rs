use ratatui::style::{Color, Modifier, Style};

// ── Color palette (inspired by Kanagawa) ──

pub const PRIMARY: Color = Color::Rgb(127, 180, 202); // blue
pub const SECONDARY: Color = Color::Rgb(152, 187, 108); // green
pub const ACCENT: Color = Color::Rgb(228, 104, 118); // red/pink
pub const MUTED: Color = Color::Rgb(114, 113, 105); // gray
pub const TEXT: Color = Color::Rgb(220, 215, 186); // warm white
pub const BG: Color = Color::Rgb(31, 31, 40); // dark bg
pub const SURFACE: Color = Color::Rgb(43, 43, 54); // slightly lighter bg
pub const YELLOW: Color = Color::Rgb(226, 194, 103);

// ── Reusable styles ──

pub fn title() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn subtitle() -> Style {
    Style::default().fg(SECONDARY)
}

pub fn normal() -> Style {
    Style::default().fg(TEXT)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn highlight() -> Style {
    Style::default().fg(BG).bg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn success() -> Style {
    Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD)
}

pub fn key_hint() -> Style {
    Style::default().fg(YELLOW)
}

pub fn border() -> Style {
    Style::default().fg(MUTED)
}

pub fn border_focused() -> Style {
    Style::default().fg(PRIMARY)
}

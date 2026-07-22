use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::theme::Theme;

pub fn render_status_bar(frame: &mut ratatui::Frame, area: Rect, text: &str) {
    let bar = format!(" {} ", text);
    frame.render_widget(
        Paragraph::new(bar).style(Theme::status_bar()),
        area,
    );
}

pub fn render_help_bar(frame: &mut ratatui::Frame, area: Rect) {
    let help = Line::from(vec![
        key_hint("j/k", "nav"),
        Span::raw(" "),
        key_hint("h/l", "fold"),
        Span::raw(" "),
        key_hint("Tab", "view"),
        Span::raw(" "),
        key_hint("Enter", "edit"),
        Span::raw(" "),
        key_hint("d", "del"),
        Span::raw(" "),
        key_hint("/", "search"),
        Span::raw(" "),
        key_hint(":w", "save"),
        Span::raw(" "),
        key_hint("q", "back"),
    ]);
    frame.render_widget(
        Paragraph::new(help).style(Style::default().bg(Theme::BG_STATUS)),
        area,
    );
}

pub fn render_edit_help_bar(frame: &mut ratatui::Frame, area: Rect) {
    let help = Line::from(vec![
        key_hint("Enter", "confirm"),
        Span::raw(" "),
        key_hint("Esc", "cancel"),
        Span::raw(" "),
        key_hint("←→", "cursor"),
        Span::raw(" "),
        key_hint("Home/End", "jump"),
    ]);
    frame.render_widget(
        Paragraph::new(help).style(Style::default().bg(Theme::BG_STATUS)),
        area,
    );
}

fn key_hint<'a>(key: &'a str, _label: &'a str) -> Span<'a> {
    Span::styled(
        format!(" {key} "),
        Style::default().fg(Color::Black).bg(Theme::ACCENT),
    )
}

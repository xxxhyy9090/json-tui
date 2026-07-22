use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::FileEntry;
use crate::theme::Theme;

pub fn render_file_picker_with_dir(
    frame: &mut Frame,
    area: Rect,
    entries: &[FileEntry],
    selected: usize,
    scan_dir: &std::path::Path,
) {
    render_file_picker_inner(frame, area, entries, selected, Some(scan_dir));
}

fn render_file_picker_inner(
    frame: &mut Frame,
    area: Rect,
    entries: &[FileEntry],
    selected: usize,
    scan_dir: Option<&std::path::Path>,
) {
    let area_height = area.height.saturating_sub(2) as usize;
    let scroll_start = if area_height > 0 {
        let half = area_height / 2;
        selected.saturating_sub(half)
    } else {
        0
    };
    let end = (scroll_start + area_height).min(entries.len());

    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No .json files found in current directory.\n\nPress q to quit.")
                .centered()
                .block(Block::default().borders(Borders::ALL)
                    .border_style(Style::default().fg(Theme::BORDER))
                    .title(" Files ")),
            area,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            "  #   File                                      Size",
            Style::default().fg(Theme::ACCENT).bold(),
        ),
    ]));

    for i in scroll_start..end {
        let entry = &entries[i];
        let is_selected = i == selected;

        let prefix = if is_selected { " ▶ " } else { "   " };
        let index = format!("{:>2}", i + 1);
        let size = format_size(entry.size);
        let max_name_len = area.width.saturating_sub(25) as usize;
        let display_name = truncate_middle(&entry.name, max_name_len);

        let mut spans = vec![
            Span::raw(prefix),
            Span::styled(index, Theme::muted()),
            Span::raw("  "),
            Span::styled(
                display_name,
                if is_selected {
                    Style::default().fg(Theme::ACCENT).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::raw("  "),
            Span::styled(size, Theme::muted()),
        ];

        if is_selected {
            let line_len: usize = spans.iter().map(|s| s.width()).sum();
            let padding = area.width.saturating_sub(line_len as u16) as usize;
            spans.push(Span::raw(" ".repeat(padding)));
            spans = spans.into_iter().map(|s| s.style(Theme::selected())).collect();
        }

        lines.push(Line::from(spans));
    }

    let title = if let Some(dir) = scan_dir {
        format!(" {}  ({} file(s)) ", dir.display(), entries.len())
    } else {
        format!(" Files ({}) ", entries.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(title);

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn truncate_middle(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    let half = max_len / 2;
    let start: String = s.chars().take(half).collect();
    let end: String = s.chars().rev().take(half).collect::<String>()
        .chars().rev().collect();
    format!("{}...{}", start, end)
}

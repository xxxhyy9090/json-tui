use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::json_tree::{JsonNode, JsonType};
use crate::theme::Theme;

pub fn render_tree_view(
    frame: &mut Frame,
    area: Rect,
    nodes: &[JsonNode],
    visible: &[usize],
    selected: usize,
    scroll_offset: u16,
) {
    let area_height = area.height.saturating_sub(2) as usize;
    let start = scroll_offset as usize;
    let end = (start + area_height).min(visible.len());

    let mut lines: Vec<Line> = Vec::new();
    for vi in start..end {
        if vi >= visible.len() {
            break;
        }
        let node_idx = visible[vi];
        let node = &nodes[node_idx];
        let is_selected = node_idx == selected;
        lines.push(render_node_line(node, is_selected));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(" Tree ");

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_node_line(node: &JsonNode, selected: bool) -> Line<'_> {
    let mut spans: Vec<Span> = Vec::new();

    // Indent
    let indent = "  ".repeat(node.depth);
    if !indent.is_empty() {
        spans.push(Span::raw(indent));
    }

    // Tree connector
    if node.depth > 0 {
        let connector = if node.is_last_child { "└─ " } else { "├─ " };
        spans.push(Span::styled(connector, Theme::muted()));
    }

    // Expand/collapse icon
    if node.is_expandable() {
        let icon = if node.expanded { "▼ " } else { "▶ " };
        spans.push(Span::styled(icon, Style::default().fg(Color::Yellow)));
    } else {
        spans.push(Span::raw("  "));
    }

    // Key
    if !node.key.is_empty() && !node.key.starts_with('[') {
        spans.push(Span::styled(format!("{}: ", node.key), Theme::key()));
    } else if node.key.starts_with('[') {
        spans.push(Span::styled(format!("{} ", node.key), Theme::muted()));
    }

    // Value with type-specific color
    let value_style = match node.value_type {
        JsonType::String => Theme::string(),
        JsonType::Number => Theme::number(),
        JsonType::Boolean => Theme::boolean(),
        JsonType::Null => Theme::null(),
        JsonType::Object | JsonType::Array => Theme::muted(),
    };
    spans.push(Span::styled(&node.value_text, value_style));

    // Selection highlight
    if selected {
        spans = spans.into_iter().map(|s| s.style(Theme::selected())).collect();
    }

    Line::from(spans)
}

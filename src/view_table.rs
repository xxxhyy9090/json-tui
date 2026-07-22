use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::json_tree::JsonType;
use crate::theme::Theme;

#[derive(Default)]
pub struct TableState {
    pub row_selected: usize,
    pub col_selected: usize,
    pub col_offset: u16,
}

pub struct TableRow {
    pub cells: Vec<(String, JsonType)>,
}

pub fn render_table_view(
    frame: &mut Frame,
    area: Rect,
    columns: &[String],
    rows: &[TableRow],
    state: &TableState,
) {
    let area_height = area.height.saturating_sub(2) as usize;
    let col = state.col_selected;

    let col_offset = if col < state.col_offset as usize {
        col as u16
    } else if col >= state.col_offset as usize + 10 {
        (col.saturating_sub(9)) as u16
    } else {
        state.col_offset
    };

    let visible_cols: Vec<(usize, &String)> = columns
        .iter()
        .enumerate()
        .skip(col_offset as usize)
        .take(10)
        .collect();

    if visible_cols.is_empty() || rows.is_empty() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new("(empty)")
                .block(Block::default().borders(Borders::ALL).title(" Table ")),
            area,
        );
        return;
    }

    // Header — highlight active column
    let header_cells: Vec<Cell> = visible_cols
        .iter()
        .map(|(ci, c)| {
            let style = if *ci == col {
                Style::default().fg(Color::Black).bg(Theme::ACCENT).bold()
            } else {
                Theme::accent().bold()
            };
            Cell::from(c.as_str()).style(style)
        })
        .collect();
    let header = Row::new(header_cells).height(1);

    // Row scroll
    let scroll_start = if area_height > 1 {
        let half = (area_height / 2).max(1);
        state.row_selected.saturating_sub(half)
    } else {
        0
    };
    let end_row = (scroll_start + area_height.saturating_sub(1)).min(rows.len());

    let display_rows: Vec<Row> = rows[scroll_start..end_row]
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let global_row = scroll_start + i;
            let is_row_sel = global_row == state.row_selected;

            let cells: Vec<Cell> = visible_cols
                .iter()
                .map(|(ci, _)| {
                    let (value, vtype) = row.cells.get(*ci).cloned()
                        .unwrap_or_else(|| (String::new(), JsonType::Null));

                    let type_style = match vtype {
                        JsonType::String => Theme::string(),
                        JsonType::Number => Theme::number(),
                        JsonType::Boolean => Theme::boolean(),
                        JsonType::Null => Theme::null(),
                        _ => Style::default(),
                    };

                    let mut s = type_style;
                    if is_row_sel && *ci == col {
                        s = Style::default().fg(Color::Black).bg(Theme::ACCENT).bold();
                    } else if is_row_sel {
                        s = s.bg(Theme::BG_SELECTED);
                    }
                    Cell::from(value).style(s)
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let widths: Vec<Constraint> = visible_cols.iter().map(|_| Constraint::Min(14)).collect();

    let table = Table::new(display_rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::BORDER))
                .title(format!(
                    " Table  {}r x {}c  ",
                    rows.len(),
                    columns.len()
                )),
        )
        .column_spacing(2);

    frame.render_widget(table, area);
}

pub fn build_table_rows(root: &serde_json::Value) -> Vec<TableRow> {
    match root {
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(|item| {
                let cells = match item {
                    serde_json::Value::Object(map) => {
                        let keys: Vec<&String> = map.keys().collect();
                        keys.iter().map(|k| (json_val_text(&map[*k]), json_val_type(&map[*k]))).collect()
                    }
                    _ => vec![(json_val_text(item), json_val_type(item))],
                };
                TableRow { cells }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn json_val_type(value: &serde_json::Value) -> JsonType {
    match value {
        serde_json::Value::Object(_) => JsonType::Object,
        serde_json::Value::Array(_) => JsonType::Array,
        serde_json::Value::String(_) => JsonType::String,
        serde_json::Value::Number(_) => JsonType::Number,
        serde_json::Value::Bool(_) => JsonType::Boolean,
        serde_json::Value::Null => JsonType::Null,
    }
}

pub fn json_val_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Object(_) => "{...}".to_string(),
        serde_json::Value::Array(a) => format!("[{} items]", a.len()),
    }
}

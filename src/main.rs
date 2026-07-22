mod app;
mod edit;
mod file_picker;
mod json_tree;
mod theme;
mod view_common;
mod view_table;
mod view_tree;

use std::{env, fs, path::PathBuf};

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use app::{App, CurrentScreen, FileEntry, ViewMode};
use edit::EditMode;
use file_picker::render_file_picker_with_dir;
use json_tree::{build_nodes, is_table_friendly, table_columns, toggle_expand, visible_nodes};
use theme::Theme;
use view_common::{render_edit_help_bar, render_help_bar, render_status_bar};
use view_table::{build_table_rows, render_table_view};
use view_tree::render_tree_view;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();

    // Determine scan directory and optional file to open
    let (scan_dir, open_target) = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.is_dir() {
            (p, None)
        } else if p.is_file() {
            (p.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(), Some(p))
        } else {
            // Path doesn't exist — treat parent as scan dir
            (p.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(), None)
        }
    } else {
        (env::current_dir()?, None)
    };

    let entries = scan_json_files(&scan_dir)?;
    let mut app = App::new_with_file_picker(entries, scan_dir);

    if let Some(path) = open_target {
        let _ = open_file(&mut app, &path);
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn scan_json_files(dir: &PathBuf) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(meta) = entry.metadata() {
                entries.push(FileEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path,
                    size: meta.len(),
                });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn open_file(app: &mut App, path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let root: serde_json::Value = serde_json::from_str(&content)?;
    let nodes = build_nodes(&root);
    let tf = is_table_friendly(&root);
    let cols = table_columns(&root);
    let rows = build_table_rows(&root);
    app.load_json(Some(path.clone()), content, root, nodes, tf, cols, rows);
    app.set_status(&format!("{}  |  Tab:view  Enter:edit  d:delete  /:search  :w:save  q:back", app.file_name));
    Ok(())
}

// ================================================================
// Event loop
// ================================================================

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    if app.screen == CurrentScreen::FilePicker {
        app.set_status("j/k:nav  Enter:open  r:refresh  q:quit");
    }
    while !app.quit {
        terminal.draw(|f| draw_ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.screen {
                    CurrentScreen::FilePicker => handle_picker_key(key.code, app)?,
                    CurrentScreen::JsonViewer => handle_viewer_key(key.code, key.modifiers, app)?,
                }
            }
        }
    }
    Ok(())
}

// ================================================================
// File picker keys
// ================================================================

fn handle_picker_key(code: KeyCode, app: &mut App) -> Result<()> {
    // Edit mode active → delegate
    if app.edit_state.mode.is_editing() {
        return handle_edit_key(code, KeyModifiers::empty(), app);
    }

    // : command prefix
    if code == KeyCode::Char(':') {
        app.edit_state.mode = EditMode::SaveAs;
        app.edit_state.input_buffer.clear();
        app.edit_state.cursor_pos = 0;
        return Ok(());
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char('j') | KeyCode::Down => {
            if app.file_picker_selected + 1 < app.file_entries.len() {
                app.file_picker_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.file_picker_selected > 0 {
                app.file_picker_selected -= 1;
            }
        }
        KeyCode::Char('g') => app.file_picker_selected = 0,
        KeyCode::Char('G') => {
            app.file_picker_selected = app.file_entries.len().saturating_sub(1);
        }
        KeyCode::Enter => {
            if !app.file_entries.is_empty() {
                let path = app.file_entries[app.file_picker_selected].path.clone();
                if let Err(e) = open_file(app, &path) {
                    app.set_status(&format!("Error: {}", e));
                }
            }
        }
        KeyCode::Char('r') => {
            app.file_entries = scan_json_files(&app.scan_dir)?;
            if app.file_picker_selected >= app.file_entries.len() {
                app.file_picker_selected = app.file_entries.len().saturating_sub(1);
            }
            app.set_status(&format!("Refreshed — {} file(s)", app.file_entries.len()));
        }
        _ => {}
    }
    Ok(())
}

// ================================================================
// Viewer keys
// ================================================================

fn handle_viewer_key(code: KeyCode, mods: KeyModifiers, app: &mut App) -> Result<()> {
    let is_ctrl = mods.contains(KeyModifiers::CONTROL);

    if app.edit_state.mode.is_editing() {
        return handle_edit_key(code, mods, app);
    }

    // : command prefix
    if code == KeyCode::Char(':') {
        app.edit_state.mode = EditMode::SaveAs;
        app.edit_state.input_buffer.clear();
        app.edit_state.cursor_pos = 0;
        return Ok(());
    }

    // / search prefix
    if code == KeyCode::Char('/') {
        app.edit_state.mode = EditMode::Search;
        app.edit_state.input_buffer.clear();
        app.edit_state.cursor_pos = 0;
        app.edit_state.error_message = None;
        return Ok(());
    }

    let visible = visible_nodes(&app.nodes);
    let visible_count = visible.len();

    match code {
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.clear_search();
            } else {
                app.back_to_picker();
                app.set_status("j/k:nav  Enter:open  r:refresh  q:quit");
            }
        }

        KeyCode::Char('q') => {
            if app.modified && !app.status_message.contains("Press again") {
                app.set_status("Unsaved changes!  s:save & exit  q:discard & exit  Esc:cancel");
            } else {
                app.back_to_picker();
            }
        }

        // Respond to save-on-quit prompt
        KeyCode::Char('s') => {
            if app.modified && app.status_message.contains("s:save") {
                save_file(app)?;
                app.set_status(&format!("Saved to {} — press q to go back", app.file_name));
            }
        }
        _ if is_ctrl && code == KeyCode::Char('s') => {
            save_file(app)?;
            app.set_status(&format!("Saved to {}", app.file_name));
        }

        // Search nav
        KeyCode::Char('n') => nav_search_next(app),
        KeyCode::Char('N') => nav_search_prev(app),

        // Movement
        KeyCode::Char('j') | KeyCode::Down => nav_down(app),
        KeyCode::Char('k') | KeyCode::Up => nav_up(app),
        KeyCode::Char('g') => nav_top(app, &visible),
        KeyCode::Char('G') => nav_bottom(app, visible_count),

        // Expand / col scroll
        KeyCode::Char('l') | KeyCode::Right => expand_or_right(app),
        KeyCode::Char('h') | KeyCode::Left => collapse_or_left(app),

        // Edit
        KeyCode::Enter => start_edit(app),
        KeyCode::Char('d') => start_delete(app),

        // View switch
        KeyCode::Tab => {
            app.toggle_view_mode();
            app.set_status(&format!("View: {}", app.view_mode.name()));
        }
        KeyCode::Char('1') => { app.view_mode = ViewMode::Tree; app.set_status("Tree view"); }
        KeyCode::Char('2') => { app.view_mode = ViewMode::Table; app.set_status("Table view"); }
        KeyCode::Char('3') => { app.view_mode = ViewMode::Auto; app.set_status("Auto view"); }

        _ => {}
    }
    Ok(())
}

fn nav_search_next(app: &mut App) {
    if !app.search_query.is_empty() {
        app.next_match();
        app.set_status(&format!("\"{}\" — {}/{}  (n/N)", app.search_query, app.current_match + 1, app.search_matches.len()));
    }
}
fn nav_search_prev(app: &mut App) {
    if !app.search_query.is_empty() {
        app.prev_match();
        app.set_status(&format!("\"{}\" — {}/{}  (n/N)", app.search_query, app.current_match + 1, app.search_matches.len()));
    }
}

// ================================================================
// Edit key handling
// ================================================================

fn handle_edit_key(code: KeyCode, _mods: KeyModifiers, app: &mut App) -> Result<()> {
    if matches!(app.edit_state.mode, EditMode::Search) {
        match code {
            KeyCode::Esc => { app.clear_search(); app.edit_state.cancel(); }
            KeyCode::Enter => {
                let q = app.edit_state.input_buffer.clone();
                app.edit_state.cancel();
                app.search(&q);
                if app.search_matches.is_empty() {
                    app.set_status(&format!("Not found: \"{}\"", q));
                } else {
                    app.set_status(&format!("\"{}\" — {}/{}  (n/N)", q, 1, app.search_matches.len()));
                }
            }
            KeyCode::Backspace => { app.edit_state.backspace(); let q = app.edit_state.input_buffer.clone(); if !q.is_empty() { app.search(&q); } }
            KeyCode::Char(c) => { app.edit_state.enter_char(c); let q = app.edit_state.input_buffer.clone(); app.search(&q); if !app.search_matches.is_empty() { app.set_status(&format!("\"{}\" — {}/{}", q, app.current_match + 1, app.search_matches.len())); } }
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Esc => app.edit_state.cancel(),
        KeyCode::Enter => confirm_edit(app)?,
        KeyCode::Backspace => app.edit_state.backspace(),
        KeyCode::Delete => app.edit_state.delete(),
        KeyCode::Left => app.edit_state.cursor_left(),
        KeyCode::Right => app.edit_state.cursor_right(),
        KeyCode::Home => app.edit_state.cursor_home(),
        KeyCode::End => app.edit_state.cursor_end(),
        KeyCode::Char(c) => app.edit_state.enter_char(c),
        _ => {}
    }
    Ok(())
}

fn confirm_edit(app: &mut App) -> Result<()> {
    match app.edit_state.mode.clone() {
        EditMode::EditTableCell { row, col } => {
            let new_val = parse_or_err(app)?;
            let col_name = &app.table_columns[col];
            let path = vec![edit::PathSegment::Index(row), edit::PathSegment::Key(col_name.clone())];
            edit::modify_value_at_path(&mut app.root, &path, new_val);
            rebuild(app);
            app.modified = true;
            app.edit_state.cancel();
            app.set_status("Cell updated");
        }
        EditMode::DeleteRow { row } => {
            if let serde_json::Value::Array(arr) = &mut app.root {
                if row < arr.len() { arr.remove(row); }
            }
            rebuild(app);
            app.modified = true;
            app.edit_state.cancel();
            app.table_state.row_selected = app.table_state.row_selected.min(app.table_rows.len().saturating_sub(1));
            app.set_status("Row deleted");
        }
        EditMode::EditValue { node_idx } => {
            let new_val = parse_or_err(app)?;
            let path = edit::build_path(&app.nodes, node_idx);
            edit::modify_value_at_path(&mut app.root, &path, new_val);
            rebuild(app);
            app.modified = true;
            app.edit_state.cancel();
            app.set_status("Value updated");
        }
        EditMode::ConfirmDelete { node_idx } => {
            let path = edit::build_path(&app.nodes, node_idx);
            if edit::delete_at_path(&mut app.root, &path) {
                rebuild(app);
                app.modified = true;
                app.edit_state.cancel();
                app.set_status("Node deleted");
            } else {
                app.edit_state.cancel();
                app.set_status("Cannot delete root");
            }
        }
        EditMode::SaveAs => {
            let cmd = app.edit_state.input_buffer.clone();
            let screen = app.screen;
            app.edit_state.cancel();
            match cmd.as_str() {
                "w" => { save_file(app)?; app.set_status(&format!("Saved to {}", app.file_name)); }
                "wq" => { save_file(app)?; app.back_to_picker(); }
                "q" | "q!" => {
                    if screen == CurrentScreen::JsonViewer { app.back_to_picker(); }
                    else { app.quit = true; }
                }
                _ if cmd.starts_with("w ") => {
                    if let Some(f) = cmd.strip_prefix("w ") { save_file_as(app, f.trim())?; }
                }
                _ if cmd.starts_with("e ") => {
                    let target = cmd.strip_prefix("e ").unwrap_or("").trim();
                    if target.is_empty() { app.set_status("Usage: :e <file or directory>"); }
                    else { open_target(app, target)?; }
                }
                _ => app.set_status(&format!("Unknown: :{}", cmd)),
            }
        }
        EditMode::Search | EditMode::Browse => {}
    }
    Ok(())
}

// ================================================================
// Navigation helpers
// ================================================================

fn nav_down(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            if app.table_state.row_selected + 1 < app.table_rows.len() {
                app.table_state.row_selected += 1;
            }
            let max = app.table_columns.len().saturating_sub(1);
            if app.table_state.col_selected > max { app.table_state.col_selected = max; }
        }
        _ => {
            let vis = visible_nodes(&app.nodes);
            let pos = app.selected_idx_in_visible(&vis);
            if pos + 1 < vis.len() { app.selected = vis[pos + 1]; update_scroll(app); }
        }
    }
    app.clear_status();
}
fn nav_up(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            if app.table_state.row_selected > 0 { app.table_state.row_selected -= 1; }
        }
        _ => {
            let vis = visible_nodes(&app.nodes);
            let pos = app.selected_idx_in_visible(&vis);
            if pos > 0 { app.selected = vis[pos - 1]; update_scroll(app); }
        }
    }
    app.clear_status();
}
fn nav_top(app: &mut App, vis: &[usize]) {
    match app.effective_view_mode() {
        ViewMode::Table => app.table_state.row_selected = 0,
        _ => { if !vis.is_empty() { app.selected = vis[0]; } app.scroll_offset = 0; }
    }
}
fn nav_bottom(app: &mut App, vc: usize) {
    match app.effective_view_mode() {
        ViewMode::Table => app.table_state.row_selected = app.table_rows.len().saturating_sub(1),
        _ => {
            let vis = visible_nodes(&app.nodes);
            if vc > 0 { app.selected = vis[vc - 1]; update_scroll(app); }
        }
    }
}
fn expand_or_right(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            let max = app.table_columns.len().saturating_sub(1);
            if app.table_state.col_selected < max {
                app.table_state.col_selected += 1;
                if app.table_state.col_selected >= app.table_state.col_offset as usize + 9 {
                    app.table_state.col_offset = (app.table_state.col_selected - 8) as u16;
                }
            }
        }
        _ => {
            if app.nodes[app.selected].is_expandable() && !app.nodes[app.selected].expanded {
                toggle_expand(&mut app.nodes, app.selected);
            }
        }
    }
}
fn collapse_or_left(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            if app.table_state.col_selected > 0 {
                app.table_state.col_selected -= 1;
                if app.table_state.col_selected < app.table_state.col_offset as usize {
                    app.table_state.col_offset = app.table_state.col_selected as u16;
                }
            }
        }
        _ => {
            if app.nodes[app.selected].is_expandable() && app.nodes[app.selected].expanded {
                toggle_expand(&mut app.nodes, app.selected);
            } else if app.nodes[app.selected].depth > 0 {
                let t = app.nodes[app.selected].depth - 1;
                for i in (0..app.selected).rev() {
                    if app.nodes[i].depth == t { app.selected = i; break; }
                }
            }
        }
    }
}
fn update_scroll(app: &mut App) {
    let vis = visible_nodes(&app.nodes);
    let pos = app.selected_idx_in_visible(&vis);
    let h: u16 = 20;
    if pos < app.scroll_offset as usize { app.scroll_offset = pos as u16; }
    else if pos >= app.scroll_offset.saturating_add(h) as usize { app.scroll_offset = (pos as u16).saturating_sub(h.saturating_sub(1)); }
    let max = vis.len().saturating_sub(h as usize) as u16;
    if app.scroll_offset > max { app.scroll_offset = max; }
}

// ================================================================
// Edit actions
// ================================================================

fn start_edit(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            if app.table_rows.is_empty() { return; }
            let r = app.table_state.row_selected;
            let c = app.table_state.col_selected;
            if c < app.table_columns.len() && r < app.table_rows.len() {
                if let Some((val, _)) = app.table_rows[r].cells.get(c) {
                    let clean = unquote(val);
                    app.edit_state.start_edit_table_cell(r, c, &clean);
                    app.set_status(&format!("Edit [{}, {}]  (Enter:confirm  Esc:cancel)", r, app.table_columns[c]));
                }
            }
        }
        _ => {
            let idx = app.selected;
            let txt = app.nodes[idx].value_text.clone();
            let clean = unquote(&txt);
            app.edit_state.start_edit_value(idx, &clean);
            app.set_status(&format!("Edit \"{}\"  (Enter:confirm  Esc:cancel)", app.nodes[idx].key));
        }
    }
}

fn unquote(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') { s[1..s.len()-1].to_string() }
    else if s.ends_with("...") { String::new() }
    else { s.to_string() }
}

fn start_delete(app: &mut App) {
    match app.effective_view_mode() {
        ViewMode::Table => {
            if app.table_rows.is_empty() { return; }
            app.edit_state.start_delete_row(app.table_state.row_selected);
            app.set_status(&format!("Delete row {}?  Enter:confirm  Esc:cancel", app.table_state.row_selected + 1));
        }
        _ => {
            if app.selected == 0 { app.set_status("Cannot delete root"); return; }
            app.edit_state.start_delete_confirm(app.selected);
            app.set_status(&format!("Delete \"{}\"?  Enter:confirm  Esc:cancel", app.nodes[app.selected].key));
        }
    }
}

fn parse_or_err(app: &mut App) -> Result<serde_json::Value> {
    match edit::parse_input_value(&app.edit_state.input_buffer) {
        Ok(v) => Ok(v),
        Err(e) => { app.edit_state.error_message = Some(e.clone()); Err(color_eyre::eyre::eyre!(e)) }
    }
}

fn rebuild(app: &mut App) {
    app.nodes = build_nodes(&app.root);
    app.table_columns = table_columns(&app.root);
    app.table_rows = build_table_rows(&app.root);
    app.table_friendly = is_table_friendly(&app.root);
    if app.selected >= app.nodes.len() { app.selected = app.nodes.len().saturating_sub(1); }
}

// ================================================================
// File I/O
// ================================================================

fn save_file(app: &mut App) -> Result<()> {
    let content = serde_json::to_string_pretty(&app.root)?;
    match &app.file_path {
        Some(p) => { fs::write(p, &content)?; app.raw_content = content; app.modified = false; }
        None => { println!("{}", content); app.raw_content = content; app.modified = false; }
    }
    Ok(())
}
/// Open a file or rescan a directory (used by :e command)
fn open_target(app: &mut App, target: &str) -> Result<()> {
    let path = PathBuf::from(target);
    if path.is_dir() {
        app.scan_dir = path;
        app.file_entries = scan_json_files(&app.scan_dir)?;
        app.file_picker_selected = 0;
        app.screen = CurrentScreen::FilePicker;
        app.set_status(&format!("Scanned {} — {} file(s)", app.scan_dir.display(), app.file_entries.len()));
    } else if path.is_file() {
        open_file(app, &path)?;
    } else {
        app.set_status(&format!("Not found: {}", target));
    }
    Ok(())
}

fn save_file_as(app: &mut App, name: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(&app.root)?;
    fs::write(name, &content)?;
    app.file_path = Some(PathBuf::from(name));
    app.file_name = name.to_string();
    app.raw_content = content;
    app.modified = false;
    app.set_status(&format!("Saved as {}", name));
    Ok(())
}

// ================================================================
// UI rendering
// ================================================================

fn draw_ui(frame: &mut Frame, app: &App) {
    match app.screen {
        CurrentScreen::FilePicker => draw_picker(frame, app),
        CurrentScreen::JsonViewer => draw_viewer(frame, app),
    }
}

fn draw_picker(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::vertical([Constraint::Min(3), Constraint::Length(1), Constraint::Length(1)]).split(area);

    render_file_picker_with_dir(frame, layout[0], &app.file_entries, app.file_picker_selected, &app.scan_dir);

    let help = Line::from(vec![
        Span::raw(" "),
        Span::styled(" j/k ", Style::default().fg(Color::Black).bg(Theme::ACCENT)),
        Span::styled("nav", Theme::muted()),
        Span::raw("  "),
        Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::styled("open", Theme::muted()),
        Span::raw("  "),
        Span::styled(" :e ", Style::default().fg(Color::Black).bg(Theme::ACCENT)),
        Span::styled("path", Theme::muted()),
        Span::raw("  "),
        Span::styled(" r ", Style::default().fg(Color::Black).bg(Theme::ACCENT)),
        Span::styled("refresh", Theme::muted()),
        Span::raw("  "),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Red)),
        Span::styled("quit", Theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(help).style(Style::default().bg(Theme::BG_STATUS)), layout[1]);
    render_status_bar(frame, layout[2], &app.status_message);
}

fn draw_viewer(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let editing = app.edit_state.mode.is_editing();

    let constraints: Vec<Constraint> = if editing {
        vec![Constraint::Min(3), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)]
    } else {
        vec![Constraint::Min(3), Constraint::Length(1), Constraint::Length(1)]
    };
    let layout = Layout::vertical(constraints).split(area);

    let (edit_area, help_area, status_area) = if editing {
        (Some(layout[1]), layout[2], layout[3])
    } else {
        (None, layout[1], layout[2])
    };

    match app.effective_view_mode() {
        ViewMode::Tree | ViewMode::Auto => {
            let vis = visible_nodes(&app.nodes);
            render_tree_view(frame, layout[0], &app.nodes, &vis, app.selected, app.scroll_offset);
        }
        ViewMode::Table => {
            render_table_view(frame, layout[0], &app.table_columns, &app.table_rows, &app.table_state);
        }
    }

    if let Some(r) = edit_area { render_edit_bar(frame, r, app); }
    if editing { render_edit_help_bar(frame, help_area); }
    else { render_help_bar(frame, help_area); }
    render_status_bar(frame, status_area, &app.status_message);
}

fn render_edit_bar(frame: &mut Frame, area: Rect, app: &App) {
    let buf = &app.edit_state.input_buffer;
    let cur = app.edit_state.cursor_pos;

    let mut spans = Vec::new();
    let label = match &app.edit_state.mode {
        EditMode::EditValue { .. } => "Edit: ",
        EditMode::EditTableCell { .. } => "Cell: ",
        EditMode::DeleteRow { .. } => "Delete? ",
        EditMode::ConfirmDelete { .. } => "Delete? ",
        EditMode::SaveAs => ":",
        EditMode::Search => "/",
        EditMode::Browse => "> ",
    };

    spans.push(Span::styled(label, Style::default().fg(Theme::ACCENT).bg(Theme::BG_STATUS)));

    if cur > 0 && cur <= buf.len() {
        spans.push(Span::styled(&buf[..cur], Style::default().fg(Color::White).bg(Theme::BG_STATUS)));
    }

    // Cursor
    let cursor_char = if cur < buf.len() { &buf[cur..next_char(&buf, cur)] } else { " " };
    spans.push(Span::styled(cursor_char, Style::default().fg(Color::Black).bg(Theme::ACCENT)));

    if cur < buf.len() {
        let after = next_char(&buf, cur);
        if after < buf.len() {
            spans.push(Span::styled(&buf[after..], Style::default().fg(Color::White).bg(Theme::BG_STATUS)));
        }
    }

    if let Some(ref err) = app.edit_state.error_message {
        spans.push(Span::styled(format!("  ✗ {}", err), Style::default().fg(Color::Red).bg(Theme::BG_STATUS)));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Theme::BG_STATUS)),
        area,
    );
}

fn next_char(s: &str, pos: usize) -> usize {
    for i in pos+1..=s.len() { if s.is_char_boundary(i) { return i; } }
    s.len()
}

use std::path::PathBuf;

use crate::edit::EditState;
use crate::json_tree::JsonNode;
use crate::view_table::{TableRow, TableState};

/// Which screen is currently displayed
#[derive(Clone, Copy, PartialEq)]
pub enum CurrentScreen {
    FilePicker,
    JsonViewer,
}

/// JSON display mode
#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Tree,
    Table,
    Auto,
}

impl ViewMode {
    pub fn name(&self) -> &str {
        match self {
            ViewMode::Tree => "Tree",
            ViewMode::Table => "Table",
            ViewMode::Auto => "Auto",
        }
    }
}

/// A single entry in the file picker
#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

/// Application state
pub struct App {
    // ---- Screen ----
    pub screen: CurrentScreen,
    pub scan_dir: PathBuf,          // directory being scanned
    pub file_picker_selected: usize,
    pub file_entries: Vec<FileEntry>,

    // ---- JSON file ----
    pub file_path: Option<PathBuf>,
    pub file_name: String,
    pub raw_content: String,
    pub root: serde_json::Value,

    // ---- Tree view state ----
    pub nodes: Vec<JsonNode>,
    pub selected: usize,
    pub scroll_offset: u16,

    // ---- Table view state ----
    pub table_columns: Vec<String>,
    pub table_rows: Vec<TableRow>,
    pub table_state: TableState,

    // ---- Edit state ----
    pub edit_state: EditState,

    // ---- Search state ----
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match: usize,

    // ---- App flags ----
    pub modified: bool,
    pub quit: bool,
    pub view_mode: ViewMode,
    pub status_message: String,
    pub table_friendly: bool,
}

impl App {
    /// Create app in file picker mode
    pub fn new_with_file_picker(file_entries: Vec<FileEntry>, scan_dir: PathBuf) -> Self {
        Self {
            screen: CurrentScreen::FilePicker,
            scan_dir,
            file_picker_selected: 0,
            file_entries,
            file_path: None,
            file_name: String::new(),
            raw_content: String::new(),
            root: serde_json::Value::Null,
            nodes: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            table_columns: Vec::new(),
            table_rows: Vec::new(),
            table_state: TableState::default(),
            edit_state: EditState::default(),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: 0,
            modified: false,
            quit: false,
            view_mode: ViewMode::Auto,
            status_message: String::new(),
            table_friendly: false,
        }
    }

    /// Load parsed JSON data and transition to viewer
    pub fn load_json(
        &mut self,
        file_path: Option<PathBuf>,
        content: String,
        root: serde_json::Value,
        nodes: Vec<JsonNode>,
        table_friendly: bool,
        table_columns: Vec<String>,
        table_rows: Vec<TableRow>,
    ) {
        self.file_name = file_path
            .as_ref()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "untitled.json".to_string());
        self.file_path = file_path;
        self.raw_content = content;
        self.root = root;
        self.nodes = nodes;
        self.selected = 0;
        self.scroll_offset = 0;
        self.table_columns = table_columns;
        self.table_rows = table_rows;
        self.table_state = TableState::default();
        self.table_friendly = table_friendly;
        self.view_mode = if table_friendly {
            ViewMode::Auto
        } else {
            ViewMode::Tree
        };
        self.screen = CurrentScreen::JsonViewer;
        self.edit_state = EditState::default();
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match = 0;
        self.modified = false;
    }

    pub fn effective_view_mode(&self) -> ViewMode {
        match self.view_mode {
            ViewMode::Auto => {
                if self.table_friendly {
                    ViewMode::Table
                } else {
                    ViewMode::Tree
                }
            }
            other => other,
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Tree => {
                if self.table_friendly {
                    ViewMode::Table
                } else {
                    ViewMode::Auto
                }
            }
            ViewMode::Table => ViewMode::Auto,
            ViewMode::Auto => ViewMode::Tree,
        };
    }

    pub fn selected_idx_in_visible(&self, visible: &[usize]) -> usize {
        visible
            .iter()
            .position(|&idx| idx == self.selected)
            .unwrap_or(0)
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    pub fn clear_status(&mut self) {
        self.status_message.clear();
    }

    /// Return to file picker screen
    pub fn back_to_picker(&mut self) {
        self.screen = CurrentScreen::FilePicker;
        self.edit_state = EditState::default();
    }

    // ---- Search ----
    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        self.current_match = 0;
        if query.is_empty() {
            return;
        }
        let lower = query.to_lowercase();
        for (i, node) in self.nodes.iter().enumerate() {
            if node.key.to_lowercase().contains(&lower)
                || node.value_text.to_lowercase().contains(&lower)
            {
                self.search_matches.push(i);
            }
        }
        if !self.search_matches.is_empty() {
            self.selected = self.search_matches[0];
            self.expand_to_node(self.selected);
        }
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        self.selected = self.search_matches[self.current_match];
        self.expand_to_node(self.selected);
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        self.selected = self.search_matches[self.current_match];
        self.expand_to_node(self.selected);
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match = 0;
    }

    fn expand_to_node(&mut self, idx: usize) {
        let mut current_depth = 0;
        for i in 0..=idx {
            if self.nodes[i].depth == current_depth
                && self.nodes[i].is_expandable()
                && i < idx
            {
                let node_depth = self.nodes[i].depth;
                let mut j = i + 1;
                while j <= idx && j < self.nodes.len() {
                    if self.nodes[j].depth <= node_depth {
                        break;
                    }
                    j += 1;
                }
                if j > idx {
                    self.nodes[i].expanded = true;
                    current_depth = self.nodes[i].depth + 1;
                }
            }
        }
    }
}

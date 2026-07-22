use serde_json::Value;

// ================================================================
// Edit mode
// ================================================================

/// Current editing mode
#[derive(Clone)]
pub enum EditMode {
    Browse,
    Search,
    EditValue { node_idx: usize },
    EditTableCell { row: usize, col: usize },
    DeleteRow { row: usize },
    SaveAs,
    ConfirmDelete { node_idx: usize },
}

impl EditMode {
    pub fn is_editing(&self) -> bool {
        !matches!(self, EditMode::Browse)
    }
}

/// Edit state (input buffer, cursor, error)
#[derive(Clone)]
pub struct EditState {
    pub mode: EditMode,
    pub input_buffer: String,
    pub cursor_pos: usize,
    pub error_message: Option<String>,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            mode: EditMode::Browse,
            input_buffer: String::new(),
            cursor_pos: 0,
            error_message: None,
        }
    }
}

impl EditState {
    pub fn start_edit_value(&mut self, node_idx: usize, current: &str) {
        self.mode = EditMode::EditValue { node_idx };
        self.input_buffer = current.to_string();
        self.cursor_pos = self.input_buffer.len();
        self.error_message = None;
    }

    pub fn start_edit_table_cell(&mut self, row: usize, col: usize, current: &str) {
        self.mode = EditMode::EditTableCell { row, col };
        self.input_buffer = current.to_string();
        self.cursor_pos = self.input_buffer.len();
        self.error_message = None;
    }

    pub fn start_delete_row(&mut self, row: usize) {
        self.mode = EditMode::DeleteRow { row };
        self.error_message = None;
    }

    pub fn start_delete_confirm(&mut self, node_idx: usize) {
        self.mode = EditMode::ConfirmDelete { node_idx };
        self.error_message = None;
    }

    pub fn cancel(&mut self) {
        self.mode = EditMode::Browse;
        self.input_buffer.clear();
        self.cursor_pos = 0;
        self.error_message = None;
    }

    pub fn enter_char(&mut self, c: char) {
        let len = c.len_utf8();
        self.input_buffer.insert(self.cursor_pos, c);
        self.cursor_pos += len;
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            let prev = prev_char_boundary(&self.input_buffer, self.cursor_pos);
            self.input_buffer.drain(prev..self.cursor_pos);
            self.cursor_pos = prev;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_pos < self.input_buffer.len() {
            let next = next_char_boundary(&self.input_buffer, self.cursor_pos);
            self.input_buffer.drain(self.cursor_pos..next);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = prev_char_boundary(&self.input_buffer, self.cursor_pos);
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.input_buffer.len() {
            self.cursor_pos = next_char_boundary(&self.input_buffer, self.cursor_pos);
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.input_buffer.len();
    }
}

// ================================================================
// Edit operations (mutate the JSON Value)
// ================================================================

/// Parse user input into a serde_json::Value.
/// Tries JSON parsing first; falls back to treating it as a plain string.
pub fn parse_input_value(input: &str) -> Result<Value, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("value cannot be empty".to_string());
    }

    // try parsing as JSON first
    match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => Ok(v),
        Err(_) => {
            // if JSON parsing fails, treat as a plain string
            Ok(Value::String(trimmed.to_string()))
        }
    }
}

/// Modify the value at a given path in the JSON tree
pub fn modify_value_at_path(root: &mut Value, path: &[PathSegment], new_value: Value) {
    if path.is_empty() {
        *root = new_value;
        return;
    }
    let target = navigate_to(root, &path[..path.len() - 1]);
    let last = path.last().unwrap();
    match (target, last) {
        (Value::Object(map), PathSegment::Key(k)) => {
            map.insert(k.clone(), new_value);
        }
        (Value::Array(arr), PathSegment::Index(i)) => {
            if *i < arr.len() {
                arr[*i] = new_value;
            }
        }
        _ => {}
    }
}

/// Delete the node at the given path
pub fn delete_at_path(root: &mut Value, path: &[PathSegment]) -> bool {
    if path.is_empty() {
        return false; // cannot delete root
    }
    let parent = navigate_to(root, &path[..path.len() - 1]);
    let last = path.last().unwrap();
    match (parent, last) {
        (Value::Object(map), PathSegment::Key(k)) => {
            map.remove(k);
            true
        }
        (Value::Array(arr), PathSegment::Index(i)) => {
            if *i < arr.len() {
                arr.remove(*i);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Navigate to the node at the given path (mutable borrow)
fn navigate_to<'a>(root: &'a mut Value, path: &[PathSegment]) -> &'a mut Value {
    let mut current = root;
    for seg in path {
        match seg {
            PathSegment::Key(k) => {
                if let Value::Object(map) = current {
                    current = map.get_mut(k).expect("key not found");
                }
            }
            PathSegment::Index(i) => {
                if let Value::Array(arr) = current {
                    current = arr.get_mut(*i).expect("index out of bounds");
                }
            }
        }
    }
    current
}

/// A single segment in a JSON path
#[derive(Debug, Clone)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

/// Reconstruct the path from root to the given node by walking backwards
/// through the flattened node list.
pub fn build_path(nodes: &[crate::json_tree::JsonNode], idx: usize) -> Vec<PathSegment> {
    let target_depth = nodes[idx].depth;

    // walk backwards to collect parent keys/indices
    let mut current_depth = target_depth;
    let mut segments = Vec::new();
    for i in (0..=idx).rev() {
        if nodes[i].depth == current_depth {
            let key = &nodes[i].key;
            if key.starts_with('[') && key.ends_with(']') {
                // array index
                let num_str = &key[1..key.len() - 1];
                if let Ok(n) = num_str.parse::<usize>() {
                    segments.push(PathSegment::Index(n));
                }
            } else if !key.is_empty() {
                segments.push(PathSegment::Key(key.clone()));
            }
            if current_depth == 0 {
                break;
            }
            current_depth -= 1;
        }
    }

    segments.reverse();
    segments
}

// ================================================================
// UTF-8 character boundary helpers
// ================================================================

/// Find the previous char boundary (byte index)
fn prev_char_boundary(s: &str, pos: usize) -> usize {
    for i in (0..pos).rev() {
        if s.is_char_boundary(i) {
            return i;
        }
    }
    0
}

/// Find the next char boundary (byte index)
fn next_char_boundary(s: &str, pos: usize) -> usize {
    for i in pos + 1..=s.len() {
        if s.is_char_boundary(i) {
            return i;
        }
    }
    s.len()
}

use serde_json::Value;
use std::collections::BTreeSet;

/// A single node in the flattened JSON tree
#[derive(Debug, Clone)]
pub struct JsonNode {
    pub key: String,            // field name (empty for root)
    pub depth: usize,           // nesting level (0 = root)
    pub expanded: bool,         // whether collapsed (Object/Array only)
    pub value_type: JsonType,
    pub value_text: String,     // display text for the value
    pub child_count: usize,     // number of children
    pub is_last_child: bool,    // whether this is the last sibling
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

impl JsonNode {
    pub fn is_expandable(&self) -> bool {
        matches!(self.value_type, JsonType::Object | JsonType::Array)
            && self.child_count > 0
    }
}

// ================================================================
// Build flattened node list
// ================================================================

/// Build a flattened list of JsonNode from a serde_json::Value
pub fn build_nodes(root: &Value) -> Vec<JsonNode> {
    let mut nodes = Vec::new();
    add_value(root, "", 0, true, &mut nodes);
    nodes
}

/// Recursively add a Value and all its descendants.
/// Returns the index of the added node in `nodes`.
fn add_value(
    value: &Value,
    key: &str,
    depth: usize,
    is_last: bool,
    nodes: &mut Vec<JsonNode>,
) -> usize {
    let my_idx = nodes.len();

    match value {
        Value::Object(map) => {
            let count = map.len();
            nodes.push(JsonNode {
                key: key.to_string(),
                depth,
                expanded: depth == 0, // expand root by default
                value_type: JsonType::Object,
                value_text: format_object_label(count),
                child_count: count,
                is_last_child: is_last,
            });

            // recursively add all children
            for (i, (k, v)) in map.iter().enumerate() {
                add_value(v, k, depth + 1, i == count - 1, nodes);
            }
        }

        Value::Array(arr) => {
            let count = arr.len();
            nodes.push(JsonNode {
                key: key.to_string(),
                depth,
                expanded: depth == 0,
                value_type: JsonType::Array,
                value_text: format_array_label(count),
                child_count: count,
                is_last_child: is_last,
            });

            for (i, v) in arr.iter().enumerate() {
                let label = format!("[{}]", i);
                add_value(v, &label, depth + 1, i == count - 1, nodes);
            }
        }

        // leaf node (no children)
        _ => {
            nodes.push(JsonNode {
                key: key.to_string(),
                depth,
                expanded: false,
                value_type: json_type(value),
                value_text: value_display(value),
                child_count: 0,
                is_last_child: is_last,
            });
        }
    }

    my_idx
}

// ================================================================
// Expand / collapse and visibility
// ================================================================

/// Toggle the expanded state of a node
pub fn toggle_expand(nodes: &mut [JsonNode], idx: usize) {
    if nodes[idx].is_expandable() {
        nodes[idx].expanded = !nodes[idx].expanded;
    }
}

/// Return indices of all visible nodes (skipping collapsed subtrees)
pub fn visible_nodes(nodes: &[JsonNode]) -> Vec<usize> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < nodes.len() {
        result.push(i);
        if nodes[i].is_expandable() && !nodes[i].expanded {
            // skip all descendants
            i = skip_descendants(nodes, i);
        } else {
            i += 1;
        }
    }
    result
}

/// Find the first node index that is NOT a descendant of `idx`
fn skip_descendants(nodes: &[JsonNode], idx: usize) -> usize {
    let depth = nodes[idx].depth;
    let mut i = idx + 1;
    while i < nodes.len() && nodes[i].depth > depth {
        i += 1;
    }
    i
}

// ================================================================
// Helpers
// ================================================================

fn json_type(value: &Value) -> JsonType {
    match value {
        Value::Object(_) => JsonType::Object,
        Value::Array(_) => JsonType::Array,
        Value::String(_) => JsonType::String,
        Value::Number(_) => JsonType::Number,
        Value::Bool(_) => JsonType::Boolean,
        Value::Null => JsonType::Null,
    }
}

fn value_display(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", truncate(s, 200)),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => String::new(), // Object/Array never appear as leaves here
        Value::Array(_) => String::new(),
    }
}

fn format_object_label(count: usize) -> String {
    match count {
        0 => "{} (empty)".to_string(),
        1 => "{} 1 field".to_string(),
        n => format!("{{}} {} fields", n),
    }
}

fn format_array_label(count: usize) -> String {
    match count {
        0 => "[] (empty)".to_string(),
        1 => "[] 1 item".to_string(),
        n => format!("[] {} items", n),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}...", s.chars().take(max_len).collect::<String>())
    } else {
        s.to_string()
    }
}

// ================================================================
// Table view helpers
// ================================================================

/// Check if this JSON is a good fit for table view.
/// Returns true when root is an array of homogeneous objects
/// (at least 50% key overlap across first 5 elements).
pub fn is_table_friendly(root: &Value) -> bool {
    match root {
        Value::Array(arr) if !arr.is_empty() => {
            let sample_size = arr.len().min(5);
            // first N elements must all be Object
            if !arr
                .iter()
                .take(sample_size)
                .all(|v| matches!(v, Value::Object(_)))
            {
                return false;
            }
            // keys from the first element
            let first_keys: BTreeSet<&String> =
                if let Value::Object(m) = &arr[0] {
                    m.keys().collect()
                } else {
                    return false;
                };
            if first_keys.is_empty() {
                return false;
            }
            // homogeneity check: each element must share >50% keys with first
            arr.iter().take(sample_size).all(|v| {
                if let Value::Object(m) = v {
                    let keys: BTreeSet<&String> = m.keys().collect();
                    let common = first_keys.intersection(&keys).count();
                    common as f64 / first_keys.len() as f64 > 0.5
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}

/// Extract column names from an array of objects
pub fn table_columns(root: &Value) -> Vec<String> {
    match root {
        Value::Array(arr) => {
            let mut keys: Vec<String> = Vec::new();
            let mut seen = BTreeSet::new();
            for item in arr.iter().take(20) {
                if let Value::Object(m) = item {
                    for k in m.keys() {
                        if seen.insert(k.clone()) {
                            keys.push(k.clone());
                        }
                    }
                }
            }
            keys
        }
        _ => Vec::new(),
    }
}

# json-tui

> Terminal JSON viewer & editor — browse, search, and modify JSON files without leaving your terminal.

Inspired by [csvlens](https://github.com/YS-L/csvlens), built with [Ratatui](https://ratatui.rs/).

![screenshot](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-brightgreen)

## Features

- 🗂️ **File picker** — scans current directory for `.json` files on launch
- 🌲 **Tree view** — navigate deeply nested objects with collapsible nodes
- 📊 **Table view** — browse arrays of objects in a spreadsheet-like grid
- 🎨 **Nord theme** — soft, eye-friendly color palette
- ✏️ **Edit values** — modify strings, numbers, booleans inline
- 🗑️ **Delete** — remove nodes (tree) or rows (table)
- 🔍 **Search** — `/` to find keys or values, `n`/`N` to jump
- 💾 **Save** — `:w` writes back to file
- 🌐 **Cross-platform** — Windows, Linux, macOS

## Quick Start

### Download binary (no Rust required)

Download the latest binary for your platform from [Releases](https://github.com/xxxhyy9090/json-tui/releases).

```bash
# Linux/macOS
chmod +x json-tui
./json-tui

# Windows
json-tui.exe
```

### Install with Cargo

```bash
cargo install json-tui
```

### Build from source

```bash
git clone https://github.com/xxxhyy9090/json-tui.git
cd json-tui
cargo build --release
./target/release/json-tui
```

## Usage

```bash
# Scan current directory for .json files
json-tui

# Scan a specific directory
json-tui ~/my-configs/

# Open a file directly
json-tui config.json
json-tui /path/to/any/file.json
```

Inside the app, use `:e <path>` to jump to any file or directory without restarting:

```
:e /home/user/other-project/
:e /tmp/some-file.json
```

### Try it locally (with test data)

The repo includes two test files for you to try:

```bash
cargo run -- test_tree.json    # nested objects & deep structures — tree view
cargo run -- test_table.json   # array of 15 crates — auto table view
```

### Keybindings

#### File Picker

| Key | Action |
|-----|--------|
| `j` / `k` , `↑` / `↓` | Navigate files |
| `Enter` | Open selected file |
| `:e <path>` | Open a file or scan a directory |
| `r` | Refresh file list |
| `q` | Quit |

#### JSON Viewer — Tree & Table

| Key | Action |
|-----|--------|
| `j` / `k` , `↑` / `↓` | Navigate up / down |
| `h` / `l` , `←` / `→` | Collapse / expand node (tree) or move column (table) |
| `g` / `G` | Jump to top / bottom |
| `Enter` | Edit value |
| `d` | Delete node (tree) or row (table) |
| `Tab` | Cycle view mode (Tree → Table → Auto) |
| `1` / `2` / `3` | Force Tree / Table / Auto view |
| `/` | Search keys & values |
| `n` / `N` | Next / previous search match |
| `:w` + `Enter` | Save file |
| `:wq` + `Enter` | Save and return to file picker |
| `:e <path>` + `Enter` | Open another file / directory |
| `q` | Return to file picker (prompts if unsaved) |
| `Esc` | Cancel search / return to file picker |

#### Editing

| Key | Action |
|-----|--------|
| Type | Enter new value |
| `Enter` | Confirm |
| `Esc` | Cancel |
| `←` / `→` | Move cursor |
| `Home` / `End` | Jump to start / end |
| `Backspace` / `Delete` | Delete character |

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) — cross-platform terminal manipulation
- [serde_json](https://crates.io/crates/serde_json) — JSON parsing

## License

MIT

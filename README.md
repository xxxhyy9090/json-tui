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
# Open file picker (scans current directory)
json-tui

# Open a specific file directly
json-tui config.json
```

### Keybindings

#### File Picker

| Key | Action |
|-----|--------|
| `j` / `k` , `↑` / `↓` | Navigate files |
| `Enter` | Open selected file |
| `r` | Refresh file list |
| `q` | Quit |

#### JSON Viewer

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
| `:q` + `Enter` | Return to file picker (discard changes) |
| `q` | Return to file picker (prompts if unsaved) |
| `Esc` | Cancel search / return to file picker |

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) — cross-platform terminal manipulation
- [serde_json](https://crates.io/crates/serde_json) — JSON parsing

## License

MIT

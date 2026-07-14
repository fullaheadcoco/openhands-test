# Rust TUI Todo

A terminal-based todo app built with [Ratatui](https://ratatui.rs/) and [Crossterm](https://github.com/crossterm-rs/crossterm).

## Features

- ✅ Add / Edit / Delete todos
- ✅ Mark as done with Space
- 🔴🟡🟢 Priority levels (High/Medium/Low) with color coding
- 🔍 Search / filter by keyword
- 💾 Auto-save to [`todos.json`](todos.json)

## Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `n` | Add new todo |
| `e` / `Enter` | Edit selected todo |
| `d` / `Delete` | Delete selected todo |
| `Space` | Toggle done |
| `p` | Cycle priority |
| `/` | Search / filter |
| `Esc` | Cancel / clear search |
| `Esc` / `q` | Quit |

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/rust-todo
```

## Project Structure

```text
src/
└── [main.rs](src/main.rs)    # All application code (single-file for simplicity)
```

Data is stored in [`todos.json`](todos.json) next to the binary.

For a detailed walkthrough, see the [docs](docs/README.md).
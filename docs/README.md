# Rust TODO — Terminal Todo App

A keyboard-driven terminal todo app built with [Ratatui](https://ratatui.rs/) and [Crossterm](https://github.com/crossterm-rs/crossterm). Todos are auto-saved to `todos.json` next to the binary.

## Quick Start

```bash
cargo build --release
./target/release/rust-todo
```

The app opens in full-screen terminal mode. No configuration needed — start typing todos immediately.

## Keyboard Shortcuts

### Normal Mode (default)

| Key | Action |
|---|---|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `n` | Enter **adding mode** (new todo) |
| `e` / `Enter` | Enter **editing mode** on selected todo |
| `Space` | Toggle done/undone on selected todo |
| `d` / `Delete` | Delete selected todo |
| `p` | Cycle priority (Low → Medium → High → Low) |
| `s` | Cycle sort mode (PRI → NEW → OLD → A-Z) |
| `/` | Enter **search mode** |
| `Ctrl+L` | Clear active search filter |
| `Esc` / `q` | Quit the application |

### Adding Mode

| Key | Action |
|---|---|
| *type text* | Compose the todo description |
| `Enter` | Save and return to Normal mode |
| `Backspace` | Delete last character |
| `Esc` | Cancel (discard input) |

### Editing Mode

| Key | Action |
|---|---|
| *type text* | Modify the todo description |
| `Enter` | Save and return to Normal mode |
| `Backspace` | Delete last character |
| `Esc` | Cancel (discard input, restore original) |

### Search Mode

| Key | Action |
|---|---|
| *type text* | Compose search query |
| `Enter` | Apply filter (case-insensitive substring match) |
| `Backspace` | Delete last character |
| `Esc` | Cancel search and clear filter |

## Features

### Priority Levels

Each todo has a **Low** (green), **Medium** (yellow), or **High** (red) priority. Press `p` in Normal mode to cycle the selected item. The priority label appears as a colored tag to the left of each todo in the list.

### Sort Modes

Press `s` to cycle through four sort orders, displayed in the title bar:

| Label | Sort Order |
|---|---|
| `PRI` | High → Medium → Low |
| `NEW` | Most recently created first |
| `OLD` | Oldest created first |
| `A-Z` | Alphabetical by todo text |

### Search / Filter

Press `/` to enter Search mode, type a keyword, then press `Enter`. The list narrows to items whose text contains the query (case-insensitive). An active filter shows a `🔍` indicator in the title bar. Press `Ctrl+L` to clear the filter and see all items again.

### Done Toggle

Press `Space` to mark a todo as complete. Completed items display with strikethrough text (`~~crossed out~~`) in dark gray. Press `Space` again to undo.

### Status Bar

The footer shows details for the currently selected item: its full text, priority level, and done/pending state. Item counts (`done/total`) appear in the list block header.

## Data

Todos are persisted automatically on every change (add, edit, delete, toggle, priority change) to `todos.json` in the working directory. The file uses pretty-printed JSON and is safe to edit by hand while the app is closed.
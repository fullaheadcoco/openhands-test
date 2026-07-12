# Rust TODO — Architecture

## Project Structure

```
rust-todo/
├── Cargo.toml          # Dependencies: ratatui, crossterm, serde, serde_json
├── Cargo.lock
├── src/
│   └── main.rs         # Single-file application (~500 lines)
├── todos.json          # Auto-generated persistence file (runtime)
├── docs/
│   ├── README.md       # Usage guide and shortcuts
│   └── ARCHITECTURE.md # This file
└── target/             # Build artifacts
```

The entire application lives in one file: `src/main.rs`. This is a deliberate choice for a small utility — all concerns (model, UI, input handling, persistence) are co-located.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `ratatui` | 0.29 | Terminal UI framework — widgets, layout, styling |
| `crossterm` | 0.28 | Terminal backend — raw mode, alternate screen, key events |
| `serde` | 1 (derive) | Serialization framework |
| `serde_json` | 1 | JSON format for persistence |

## Data Model

### `Todo` struct

```rust
struct Todo {
    text: String,       // The todo description
    done: bool,         // Completion status
    priority: Priority, // High / Medium / Low
    created_at: u64,    // Unix timestamp (seconds)
}
```

`created_at` defaults to `now()` via `#[serde(default = "default_created_at")]`, providing backward compatibility with older `todos.json` files that lack this field.

### `Priority` enum

```rust
enum Priority { High, Medium, Low }
```

Implements:
- `next()` — cycles Low → Medium → High → Low
- `color()` — maps to Ratatui `Color` (Red / Yellow / Green)
- `label()` — returns 4-char display string (`HIGH`, `MED `, `LOW `)

### `SortMode` enum

```rust
enum SortMode { Priority, Newest, Oldest, Alpha }
```

Implements:
- `next()` — cycles through all four modes
- `label()` — returns 3-char display string (`PRI`, `NEW`, `OLD`, `A-Z`)

### `TodoList` struct

```rust
struct TodoList {
    items: Vec<Todo>,
}
```

Wraps the vector and provides `load()` / `save()` that read/write `todos.json` via `serde_json`. Load failures (missing file, parse error) silently fall back to an empty list.

## Application State

### `App` struct

The central state object holding all mutable application data:

| Field | Type | Purpose |
|---|---|---|
| `todos` | `TodoList` | Master list of all todos |
| `list_state` | `ListState` | Which item is highlighted in the UI |
| `input` | `String` | Text buffer for add/edit/search input |
| `input_mode` | `InputMode` | Current interaction mode |
| `search_query` | `String` | Active filter (empty = show all) |
| `filtered_indices` | `Vec<usize>` | Indices into `todos.items` after search/sort |
| `sort_mode` | `SortMode` | Current sort order |
| `should_quit` | `bool` | Exit flag for the main loop |

### `InputMode` enum

```rust
enum InputMode { Normal, Adding, Editing(usize), Searching }
```

The mode determines how key events are interpreted. `Editing(usize)` carries the real index into `todos.items` so the edit can be committed back to the correct item.

### Indirection: `filtered_indices`

The UI displays items through an indirection layer:

```
todos.items (real)  →  filtered_indices (sorted, filtered)  →  list_state (cursor)
```

Operations work in this order:
1. **Search** filters `todos.items` by substring match → populates `filtered_indices`
2. **Sort** reorders `filtered_indices` according to `sort_mode`
3. **Cursor** tracks position within `filtered_indices` via `list_state`

This keeps the real list stable (important for `Editing(usize)` which stores a real index) while allowing both filtering and sorting to be applied independently.

## UI Layout

The terminal is divided into three vertical regions:

```
┌──────────────────────────────────────────┐
│ 📋 Rust TODO | PRI | n:add e:edit ...    │  ← Title bar (1 row)
├──────────────────────────────────────────┤
│                                          │
│  HIGH ✓ Buy groceries                   │  ← Todo list (fills remaining space)
│  MED  ☐ Write docs                      │
│  LOW  ☐ Walk the dog                    │
│                                          │
├──────────────────────────────────────────┤
│ Selected: "Write docs" | MED | Pending   │  ← Footer (3 rows)
└──────────────────────────────────────────┘
```

The footer content changes based on `input_mode`:
- **Normal**: Shows details of the selected item
- **Adding**: Text input with yellow styling
- **Editing**: Text input with green styling, pre-filled with existing text
- **Searching**: Text input with magenta styling

## Event Loop

```
main()
  │
  ├─ enable_raw_mode()          ← crossterm: raw input, no line buffering
  ├─ EnterAlternateScreen       ← crossterm: switch to full-screen buffer
  ├─ App::new()                 ← loads todos.json, initializes state
  │
  └─ while !should_quit:
       ├─ terminal.draw(ui)     ← ratatui: render frame
       └─ event::read()         ← crossterm: block until key press
            └─ handle_key()     ← dispatch based on input_mode + key
```

### Key Dispatch

`handle_key()` is a two-level match:
1. Outer match on `app.input_mode` (Normal / Adding / Editing / Searching)
2. Inner match on the specific `KeyCode`

This creates four distinct keymaps. The same physical key (e.g. `Enter`) can have different meanings in different modes.

## Persistence

- **Format**: Pretty-printed JSON via `serde_json::to_string_pretty()`
- **Trigger**: Every mutating operation calls `app.save()` immediately (eager persistence, not on-exit)
- **Load**: On startup via `App::new()` → `TodoList::load()`, which returns an empty list on any error
- **Backward compatibility**: New fields (`created_at`) use `#[serde(default)]` so old JSON files load without issues

## Key Design Decisions

1. **Single-file architecture**: For a tool of this size, splitting into modules adds complexity without benefit. All types and functions are visible in one place.

2. **Eager persistence**: Saving on every mutation is simpler than tracking dirty state. The JSON file is small enough that write overhead is negligible.

3. **Filtered-indices pattern**: Rather than creating temporary filtered/sorted copies, the app keeps the original list intact and maps through indices. This avoids duplicating `Todo` data and keeps the `Editing(usize)` real-index invariant sound.

4. **Mode-based input handling**: Four input modes with distinct keymaps provide a modal-editing feel (inspired by vim) while keeping the dispatch logic readable — each mode is a flat `match` arm.
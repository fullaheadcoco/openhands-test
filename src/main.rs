use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, stdout},
    time::{SystemTime, UNIX_EPOCH},
};

const SAVE_FILE: &str = "todos.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq)]
enum SortMode {
    Priority,
    Newest,
    Oldest,
    Alpha,
    Tag,
}

impl SortMode {
    fn next(&self) -> Self {
        match self {
            SortMode::Priority => SortMode::Newest,
            SortMode::Newest => SortMode::Oldest,
            SortMode::Oldest => SortMode::Alpha,
            SortMode::Alpha => SortMode::Tag,
            SortMode::Tag => SortMode::Priority,
        }
    }
    fn label(&self) -> &str {
        match self {
            SortMode::Priority => "PRI",
            SortMode::Newest => "NEW",
            SortMode::Oldest => "OLD",
            SortMode::Alpha => "A-Z",
            SortMode::Tag => "TAG",
        }
    }
}

impl Priority {
    fn next(&self) -> Self {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Low,
        }
    }
    fn color(&self) -> Color {
        match self {
            Priority::High => Color::Red,
            Priority::Medium => Color::Yellow,
            Priority::Low => Color::Green,
        }
    }
    fn label(&self) -> &str {
        match self {
            Priority::High => "HIGH",
            Priority::Medium => "MED ",
            Priority::Low => "LOW ",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    text: String,
    done: bool,
    priority: Priority,
    #[serde(default = "default_created_at")]
    created_at: u64,
    #[serde(default)]
    tags: Vec<String>,
}

fn default_created_at() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TodoList {
    items: Vec<Todo>,
}

impl TodoList {
    fn load() -> Self {
        fs::read_to_string(SAVE_FILE)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| TodoList { items: vec![] })
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(SAVE_FILE, json);
        }
    }
}

enum InputMode {
    Normal,
    Adding,
    Editing(usize),
    Searching,
    Tagging(usize),
}

struct App {
    todos: TodoList,
    list_state: ListState,
    input: String,
    input_mode: InputMode,
    search_query: String,
    filtered_indices: Vec<usize>,
    sort_mode: SortMode,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let todos = TodoList::load();
        let mut list_state = ListState::default();
        if !todos.items.is_empty() {
            list_state.select(Some(0));
        }
        let filtered_indices: Vec<usize> = (0..todos.items.len()).collect();
        let mut app = App {
            todos,
            list_state,
            input: String::new(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_indices,
            sort_mode: SortMode::Newest,
            should_quit: false,
        };
        app.sort_filtered();
        if !app.filtered_indices.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    fn save(&self) {
        self.todos.save();
    }

    fn add_todo(&mut self) {
        let text = self.input.trim().to_string();
        if !text.is_empty() {
            self.todos.items.push(Todo {
                text,
                done: false,
                priority: Priority::Medium,
                created_at: now_secs(),
                tags: vec![],
            });
            self.refresh_filter();
            self.list_state
                .select(Some(self.filtered_indices.len().saturating_sub(1)));
            self.save();
        }
        self.input.clear();
        self.input_mode = InputMode::Normal;
    }

    fn delete_todo(&mut self) {
        if let Some(filtered_idx) = self.list_state.selected() {
            if let Some(&real_idx) = self.filtered_indices.get(filtered_idx) {
                self.todos.items.remove(real_idx);
                self.refresh_filter();
                let new_len = self.filtered_indices.len();
                if new_len > 0 && filtered_idx >= new_len {
                    self.list_state.select(Some(new_len.saturating_sub(1)));
                } else if new_len == 0 {
                    self.list_state.select(None);
                }
                self.save();
            }
        }
    }

    fn toggle_done(&mut self) {
        if let Some(filtered_idx) = self.list_state.selected() {
            if let Some(&real_idx) = self.filtered_indices.get(filtered_idx) {
                self.todos.items[real_idx].done = !self.todos.items[real_idx].done;
                self.save();
            }
        }
    }

    fn cycle_priority(&mut self) {
        if let Some(filtered_idx) = self.list_state.selected() {
            if let Some(&real_idx) = self.filtered_indices.get(filtered_idx) {
                self.todos.items[real_idx].priority = self.todos.items[real_idx].priority.next();
                self.save();
            }
        }
    }

    fn confirm_edit(&mut self) {
        if let InputMode::Editing(idx) = self.input_mode {
            let text = self.input.trim().to_string();
            if !text.is_empty() {
                self.todos.items[idx].text = text;
                self.save();
            }
        }
        self.input.clear();
        self.input_mode = InputMode::Normal;
    }

    fn start_tagging(&mut self, real_idx: usize) {
        self.input = self.todos.items[real_idx].tags.join(", ");
        self.input_mode = InputMode::Tagging(real_idx);
    }

    fn confirm_tags(&mut self) {
        if let InputMode::Tagging(idx) = self.input_mode {
            let tags: Vec<String> = self
                .input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            self.todos.items[idx].tags = tags;
            self.save();
        }
        self.input.clear();
        self.input_mode = InputMode::Normal;
    }

    fn refresh_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_indices = self
            .todos
            .items
            .iter()
            .enumerate()
            .filter(|(_, t)| t.text.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        self.sort_filtered();
    }

    fn sort_filtered(&mut self) {
        match self.sort_mode {
            SortMode::Priority => {
                self.filtered_indices
                    .sort_by_key(|&i| match self.todos.items[i].priority {
                        Priority::High => 0,
                        Priority::Medium => 1,
                        Priority::Low => 2,
                    });
            }
            SortMode::Newest => {
                self.filtered_indices.sort_by(|&a, &b| {
                    self.todos.items[b]
                        .created_at
                        .cmp(&self.todos.items[a].created_at)
                });
            }
            SortMode::Oldest => {
                self.filtered_indices.sort_by(|&a, &b| {
                    self.todos.items[a]
                        .created_at
                        .cmp(&self.todos.items[b].created_at)
                });
            }
            SortMode::Alpha => {
                self.filtered_indices.sort_by(|&a, &b| {
                    self.todos.items[a]
                        .text
                        .to_lowercase()
                        .cmp(&self.todos.items[b].text.to_lowercase())
                });
            }
            SortMode::Tag => {
                self.filtered_indices.sort_by(|&a, &b| {
                    let ta = self.todos.items[a]
                        .tags
                        .first()
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    let tb = self.todos.items[b]
                        .tags
                        .first()
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    ta.cmp(&tb)
                });
            }
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.sort_filtered();
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn apply_search(&mut self) {
        let q = self.input.trim().to_lowercase();
        self.search_query = q;
        self.input.clear();
        self.input_mode = InputMode::Normal;
        self.refresh_filter();
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new =
            (current as i32 + delta).clamp(0, self.filtered_indices.len() as i32 - 1) as usize;
        self.list_state.select(Some(new));
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut app = App::new();
    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?;

    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key(key.code, &mut app);
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_key(key: KeyCode, app: &mut App) {
    match &app.input_mode {
        InputMode::Normal => match key {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => app.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => app.move_selection(-1),
            KeyCode::Char('n') => app.input_mode = InputMode::Adding,
            KeyCode::Char('d') | KeyCode::Delete => app.delete_todo(),
            KeyCode::Char(' ') => app.toggle_done(),
            KeyCode::Char('p') => app.cycle_priority(),
            KeyCode::Char('e') | KeyCode::Enter => {
                if let Some(filtered_idx) = app.list_state.selected() {
                    if let Some(&real_idx) = app.filtered_indices.get(filtered_idx) {
                        app.input = app.todos.items[real_idx].text.clone();
                        app.input_mode = InputMode::Editing(real_idx);
                    }
                }
            }
            KeyCode::Char('/') => {
                app.input_mode = InputMode::Searching;
                app.input.clear();
            }
            KeyCode::Char('s') => app.cycle_sort(),
            KeyCode::Char('t') => {
                if let Some(filtered_idx) = app.list_state.selected() {
                    if let Some(&real_idx) = app.filtered_indices.get(filtered_idx) {
                        app.start_tagging(real_idx);
                    }
                }
            }
            KeyCode::Char('l') => {
                // Ctrl+L: clear search
                app.search_query.clear();
                app.refresh_filter();
                if !app.filtered_indices.is_empty() {
                    app.list_state.select(Some(0));
                }
            }
            _ => {}
        },
        InputMode::Adding => match key {
            KeyCode::Esc => {
                app.input.clear();
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => app.add_todo(),
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            _ => {}
        },
        InputMode::Editing(_) => match key {
            KeyCode::Esc => {
                app.input.clear();
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => app.confirm_edit(),
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            _ => {}
        },
        InputMode::Searching => match key {
            KeyCode::Esc => {
                app.input.clear();
                app.search_query.clear();
                app.refresh_filter();
                if !app.filtered_indices.is_empty() {
                    app.list_state.select(Some(0));
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => app.apply_search(),
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            _ => {}
        },
        InputMode::Tagging(_) => match key {
            KeyCode::Esc => {
                app.input.clear();
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => app.confirm_tags(),
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            _ => {}
        },
    }
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(3),
    ])
    .split(area);

    // Title bar
    let title = if app.search_query.is_empty() {
        Line::from(vec![
            Span::styled("📋 Rust TODO", Style::new().bold().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::styled(app.sort_mode.label(), Style::new().fg(Color::Blue)),
            Span::raw(" | "),
            Span::styled("n", Style::new().fg(Color::Yellow)),
            Span::raw(":add "),
            Span::styled("e", Style::new().fg(Color::Yellow)),
            Span::raw(":edit "),
            Span::styled("d", Style::new().fg(Color::Yellow)),
            Span::raw(":del "),
            Span::styled("s", Style::new().fg(Color::Yellow)),
            Span::raw(":sort "),
            Span::styled("\u{2423}", Style::new().fg(Color::Yellow)),
            Span::raw(":done "),
            Span::styled("p", Style::new().fg(Color::Yellow)),
            Span::raw(":pri "),
            Span::styled("t", Style::new().fg(Color::Yellow)),
            Span::raw(":tag "),
            Span::styled("/", Style::new().fg(Color::Yellow)),
            Span::raw(":find "),
            Span::styled("q", Style::new().fg(Color::Yellow)),
            Span::raw(":quit"),
        ])
    } else {
        Line::from(vec![
            Span::styled("📋 Rust TODO", Style::new().bold().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::styled(app.sort_mode.label(), Style::new().fg(Color::Blue)),
            Span::raw(" | 🔍 "),
            Span::styled(&app.search_query, Style::new().fg(Color::Magenta)),
            Span::raw(" | "),
            Span::styled("Esc", Style::new().fg(Color::Yellow)),
            Span::raw(":clear"),
        ])
    };
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Todo list
    let total = app.todos.items.len();
    let done_count = app.todos.items.iter().filter(|t| t.done).count();
    let list_title = format!(
        " Todos ({}/{}) {}",
        done_count,
        total,
        if app.search_query.is_empty() {
            String::new()
        } else {
            format!("filtered: {}", app.filtered_indices.len())
        }
    );

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&i| {
            let todo = &app.todos.items[i];
            let marker = if todo.done { "✓" } else { "☐" };
            let text = if todo.done {
                Span::styled(
                    format!("{} ~~{}~~", marker, todo.text),
                    Style::new().fg(Color::DarkGray).crossed_out(),
                )
            } else {
                Span::styled(
                    format!("{} {}", marker, todo.text),
                    Style::new().fg(Color::White),
                )
            };
            let priority_label = Span::styled(
                todo.priority.label(),
                Style::new().fg(todo.priority.color()),
            );
            let mut spans = vec![priority_label, Span::raw(" ")];
            if !todo.tags.is_empty() {
                spans.push(Span::styled(
                    format!("[{}] ", todo.tags.join(", ")),
                    Style::new().fg(Color::Blue),
                ));
            }
            spans.push(text);
            Line::from(spans)
        })
        .map(ListItem::new)
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title(list_title))
        .highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, chunks[1], &mut app.list_state.clone());

    // Footer / input area
    match &app.input_mode {
        InputMode::Normal => {
            let status = if app.todos.items.is_empty() {
                "No todos yet. Press 'n' to add one!".to_string()
            } else if let Some(idx) = app.list_state.selected() {
                if let Some(&real_idx) = app.filtered_indices.get(idx) {
                    let t = &app.todos.items[real_idx];
                    let tag_str = if t.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" | tags: [{}]", t.tags.join(", "))
                    };
                    format!(
                        "Selected: \"{}\" | {} | {}{}",
                        t.text,
                        t.priority.label(),
                        if t.done { "Done" } else { "Pending" },
                        tag_str
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let footer = Paragraph::new(status).block(Block::bordered().title(" Status ").dim());
            f.render_widget(footer, chunks[2]);
        }
        InputMode::Adding => {
            let input = Paragraph::new(app.input.as_str())
                .block(Block::bordered().title(" New Todo (Enter: save, Esc: cancel) "))
                .style(Style::new().fg(Color::Yellow));
            f.render_widget(input, chunks[2]);
        }
        InputMode::Editing(_) => {
            let input = Paragraph::new(app.input.as_str())
                .block(Block::bordered().title(" Edit (Enter: save, Esc: cancel) "))
                .style(Style::new().fg(Color::Green));
            f.render_widget(input, chunks[2]);
        }
        InputMode::Searching => {
            let input = Paragraph::new(app.input.as_str())
                .block(Block::bordered().title(" Search (Enter: apply, Esc: cancel) "))
                .style(Style::new().fg(Color::Magenta));
            f.render_widget(input, chunks[2]);
        }
        InputMode::Tagging(_) => {
            let input = Paragraph::new(app.input.as_str())
                .block(
                    Block::bordered().title(" Tags (comma-separated, Enter: save, Esc: cancel) "),
                )
                .style(Style::new().fg(Color::Blue));
            f.render_widget(input, chunks[2]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_todo(text: &str, priority: Priority, done: bool, created_at: u64) -> Todo {
        Todo {
            text: text.to_string(),
            done,
            priority,
            created_at,
            tags: vec![],
        }
    }

    fn make_todo_with_tags(
        text: &str,
        priority: Priority,
        done: bool,
        created_at: u64,
        tags: Vec<String>,
    ) -> Todo {
        Todo {
            text: text.to_string(),
            done,
            priority,
            created_at,
            tags,
        }
    }

    #[test]
    fn test_todo_creation() {
        let todo = make_todo("Test todo", Priority::Medium, false, 1000);
        assert_eq!(todo.text, "Test todo");
        assert!(!todo.done);
        assert!(matches!(todo.priority, Priority::Medium));
        assert_eq!(todo.created_at, 1000);
        assert!(todo.tags.is_empty());
    }

    #[test]
    fn test_priority_cycle() {
        assert!(matches!(Priority::Low.next(), Priority::Medium));
        assert!(matches!(Priority::Medium.next(), Priority::High));
        assert!(matches!(Priority::High.next(), Priority::Low));
    }

    #[test]
    fn test_priority_color() {
        assert_eq!(Priority::High.color(), Color::Red);
        assert_eq!(Priority::Medium.color(), Color::Yellow);
        assert_eq!(Priority::Low.color(), Color::Green);
    }

    #[test]
    fn test_priority_label() {
        assert_eq!(Priority::High.label(), "HIGH");
        assert_eq!(Priority::Medium.label(), "MED ");
        assert_eq!(Priority::Low.label(), "LOW ");
    }

    #[test]
    fn test_toggle_done() {
        let mut todo = make_todo("Test", Priority::Low, false, 0);
        assert!(!todo.done);
        todo.done = !todo.done;
        assert!(todo.done);
        todo.done = !todo.done;
        assert!(!todo.done);
    }

    #[test]
    fn test_add_remove_from_list() {
        let mut list = TodoList { items: vec![] };
        assert!(list.items.is_empty());

        list.items
            .push(make_todo("First", Priority::High, false, 1));
        assert_eq!(list.items.len(), 1);

        list.items
            .push(make_todo("Second", Priority::Low, false, 2));
        assert_eq!(list.items.len(), 2);

        list.items.remove(0);
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].text, "Second");
    }

    #[test]
    fn test_sort_mode_cycle() {
        assert!(matches!(SortMode::Priority.next(), SortMode::Newest));
        assert!(matches!(SortMode::Newest.next(), SortMode::Oldest));
        assert!(matches!(SortMode::Oldest.next(), SortMode::Alpha));
        assert!(matches!(SortMode::Alpha.next(), SortMode::Tag));
        assert!(matches!(SortMode::Tag.next(), SortMode::Priority));
    }

    #[test]
    fn test_sort_mode_label() {
        assert_eq!(SortMode::Priority.label(), "PRI");
        assert_eq!(SortMode::Newest.label(), "NEW");
        assert_eq!(SortMode::Oldest.label(), "OLD");
        assert_eq!(SortMode::Alpha.label(), "A-Z");
        assert_eq!(SortMode::Tag.label(), "TAG");
    }

    #[test]
    fn test_todo_with_tags() {
        let todo = make_todo_with_tags(
            "Test",
            Priority::High,
            false,
            1,
            vec!["bug".to_string(), "urgent".to_string()],
        );
        assert_eq!(todo.tags.len(), 2);
        assert_eq!(todo.tags[0], "bug");
        assert_eq!(todo.tags[1], "urgent");
    }

    #[test]
    fn test_todo_empty_tags() {
        let todo = make_todo("Test", Priority::Medium, false, 1);
        assert!(todo.tags.is_empty());
    }

    #[test]
    fn test_todo_serde_tags_default() {
        let json = r#"{"text":"Test","done":false,"priority":"High","created_at":100}"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert!(todo.tags.is_empty());
    }

    #[test]
    fn test_todo_serde_with_tags() {
        let todo = make_todo_with_tags("Test", Priority::High, false, 100, vec!["bug".to_string()]);
        let json = serde_json::to_string(&todo).unwrap();
        let parsed: Todo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tags.len(), 1);
        assert_eq!(parsed.tags[0], "bug");
    }

    #[test]
    fn test_confirm_tags() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Medium, false, 1)];
        app.filtered_indices = vec![0];
        app.list_state.select(Some(0));

        app.input = "bug, urgent, feature".to_string();
        app.input_mode = InputMode::Tagging(0);
        app.confirm_tags();

        assert_eq!(app.todos.items[0].tags.len(), 3);
        assert_eq!(app.todos.items[0].tags[0], "bug");
        assert_eq!(app.todos.items[0].tags[1], "urgent");
        assert_eq!(app.todos.items[0].tags[2], "feature");
        assert!(matches!(app.input_mode, InputMode::Normal));
    }

    #[test]
    fn test_confirm_tags_empty_and_whitespace() {
        let mut app = App::new();
        app.todos.items = vec![make_todo_with_tags(
            "Test",
            Priority::Medium,
            false,
            1,
            vec!["old".to_string()],
        )];
        app.filtered_indices = vec![0];

        app.input = " , , ".to_string();
        app.input_mode = InputMode::Tagging(0);
        app.confirm_tags();

        assert!(app.todos.items[0].tags.is_empty());
    }

    #[test]
    fn test_confirm_tags_trims_whitespace() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Medium, false, 1)];
        app.filtered_indices = vec![0];

        app.input = "  bug ,  urgent  ".to_string();
        app.input_mode = InputMode::Tagging(0);
        app.confirm_tags();

        assert_eq!(app.todos.items[0].tags[0], "bug");
        assert_eq!(app.todos.items[0].tags[1], "urgent");
    }

    #[test]
    fn test_start_tagging() {
        let mut app = App::new();
        app.todos.items = vec![make_todo_with_tags(
            "Test",
            Priority::Medium,
            false,
            1,
            vec!["rust".to_string()],
        )];
        app.filtered_indices = vec![0];

        app.start_tagging(0);
        assert_eq!(app.input, "rust");
        assert!(matches!(app.input_mode, InputMode::Tagging(0)));
    }

    #[test]
    fn test_start_tagging_empty() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Medium, false, 1)];
        app.filtered_indices = vec![0];

        app.start_tagging(0);
        assert_eq!(app.input, "");
        assert!(matches!(app.input_mode, InputMode::Tagging(0)));
    }

    #[test]
    fn test_sort_by_tag() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo_with_tags("B", Priority::Medium, false, 1, vec!["z".to_string()]),
            make_todo_with_tags("A", Priority::Medium, false, 2, vec!["a".to_string()]),
            make_todo_with_tags("C", Priority::Medium, false, 3, vec![]),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.sort_mode = SortMode::Tag;
        app.sort_filtered();

        // Items without tags come first (empty string), then "a", then "z"
        let first = &app.todos.items[app.filtered_indices[0]];
        let second = &app.todos.items[app.filtered_indices[1]];
        let third = &app.todos.items[app.filtered_indices[2]];

        assert!(first.tags.is_empty());
        assert_eq!(second.tags[0], "a");
        assert_eq!(third.tags[0], "z");
    }

    #[test]
    fn test_sort_by_priority() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("Low prio", Priority::Low, false, 1),
            make_todo("High prio", Priority::High, false, 2),
            make_todo("Med prio", Priority::Medium, false, 3),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.sort_mode = SortMode::Priority;
        app.sort_filtered();

        let first = &app.todos.items[app.filtered_indices[0]];
        let second = &app.todos.items[app.filtered_indices[1]];
        let third = &app.todos.items[app.filtered_indices[2]];

        assert!(matches!(first.priority, Priority::High));
        assert!(matches!(second.priority, Priority::Medium));
        assert!(matches!(third.priority, Priority::Low));
    }

    #[test]
    fn test_sort_by_newest() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("Old", Priority::Medium, false, 1),
            make_todo("New", Priority::Medium, false, 100),
            make_todo("Mid", Priority::Medium, false, 50),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.sort_mode = SortMode::Newest;
        app.sort_filtered();

        let sorted: Vec<u64> = app
            .filtered_indices
            .iter()
            .map(|&i| app.todos.items[i].created_at)
            .collect();
        assert_eq!(sorted, vec![100, 50, 1]);
    }

    #[test]
    fn test_sort_by_oldest() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("New", Priority::Medium, false, 100),
            make_todo("Old", Priority::Medium, false, 1),
            make_todo("Mid", Priority::Medium, false, 50),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.sort_mode = SortMode::Oldest;
        app.sort_filtered();

        let sorted: Vec<u64> = app
            .filtered_indices
            .iter()
            .map(|&i| app.todos.items[i].created_at)
            .collect();
        assert_eq!(sorted, vec![1, 50, 100]);
    }

    #[test]
    fn test_sort_by_alpha() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("banana", Priority::Medium, false, 1),
            make_todo("apple", Priority::Medium, false, 2),
            make_todo("Cherry", Priority::Medium, false, 3),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.sort_mode = SortMode::Alpha;
        app.sort_filtered();

        let sorted: Vec<&str> = app
            .filtered_indices
            .iter()
            .map(|&i| app.todos.items[i].text.as_str())
            .collect();
        assert_eq!(sorted, vec!["apple", "banana", "Cherry"]);
    }

    #[test]
    fn test_cycle_sort() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Medium, false, 1)];
        app.filtered_indices = vec![0];
        app.list_state.select(Some(0));

        assert!(matches!(app.sort_mode, SortMode::Newest));
        app.cycle_sort();
        assert!(matches!(app.sort_mode, SortMode::Oldest));
        app.cycle_sort();
        assert!(matches!(app.sort_mode, SortMode::Alpha));
        app.cycle_sort();
        assert!(matches!(app.sort_mode, SortMode::Tag));
        app.cycle_sort();
        assert!(matches!(app.sort_mode, SortMode::Priority));
        app.cycle_sort();
        assert!(matches!(app.sort_mode, SortMode::Newest));
    }

    #[test]
    fn test_cycle_priority() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Low, false, 1)];
        app.filtered_indices = vec![0];
        app.list_state.select(Some(0));

        app.cycle_priority();
        assert!(matches!(app.todos.items[0].priority, Priority::Medium));
        app.cycle_priority();
        assert!(matches!(app.todos.items[0].priority, Priority::High));
        app.cycle_priority();
        assert!(matches!(app.todos.items[0].priority, Priority::Low));
    }

    #[test]
    fn test_toggle_done_in_app() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Test", Priority::Medium, false, 1)];
        app.filtered_indices = vec![0];
        app.list_state.select(Some(0));

        assert!(!app.todos.items[0].done);
        app.toggle_done();
        assert!(app.todos.items[0].done);
        app.toggle_done();
        assert!(!app.todos.items[0].done);
    }

    #[test]
    fn test_delete_todo() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("First", Priority::Medium, false, 1),
            make_todo("Second", Priority::Medium, false, 2),
        ];
        app.filtered_indices = vec![0, 1];
        app.list_state.select(Some(0));

        assert_eq!(app.todos.items.len(), 2);
        app.delete_todo();
        assert_eq!(app.todos.items.len(), 1);
        assert_eq!(app.todos.items[0].text, "Second");
    }

    #[test]
    fn test_add_todo() {
        let mut app = App::new();
        app.input = "New task".to_string();
        app.input_mode = InputMode::Adding;
        app.add_todo();

        assert_eq!(app.todos.items.len(), 1);
        assert_eq!(app.todos.items[0].text, "New task");
        assert!(!app.todos.items[0].done);
        assert!(matches!(app.todos.items[0].priority, Priority::Medium));
        assert!(app.todos.items[0].tags.is_empty());
    }

    #[test]
    fn test_add_todo_empty_input() {
        let mut app = App::new();
        app.input = "   ".to_string();
        app.input_mode = InputMode::Adding;
        app.add_todo();

        assert!(app.todos.items.is_empty());
    }

    #[test]
    fn test_confirm_edit() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Old text", Priority::Medium, false, 1)];
        app.input = "New text".to_string();
        app.input_mode = InputMode::Editing(0);
        app.confirm_edit();

        assert_eq!(app.todos.items[0].text, "New text");
        assert!(matches!(app.input_mode, InputMode::Normal));
    }

    #[test]
    fn test_confirm_edit_empty_input() {
        let mut app = App::new();
        app.todos.items = vec![make_todo("Old text", Priority::Medium, false, 1)];
        app.input = "   ".to_string();
        app.input_mode = InputMode::Editing(0);
        app.confirm_edit();

        assert_eq!(app.todos.items[0].text, "Old text");
    }

    #[test]
    fn test_refresh_filter() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("apple", Priority::Medium, false, 1),
            make_todo("banana", Priority::Medium, false, 2),
            make_todo("cherry", Priority::Medium, false, 3),
        ];
        app.search_query = "a".to_string();
        app.refresh_filter();
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_refresh_filter_no_match() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("apple", Priority::Medium, false, 1),
            make_todo("banana", Priority::Medium, false, 2),
        ];
        app.search_query = "xyz".to_string();
        app.refresh_filter();
        assert!(app.filtered_indices.is_empty());
    }

    #[test]
    fn test_move_selection() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("A", Priority::Medium, false, 1),
            make_todo("B", Priority::Medium, false, 2),
            make_todo("C", Priority::Medium, false, 3),
        ];
        app.filtered_indices = vec![0, 1, 2];
        app.list_state.select(Some(1));

        app.move_selection(-1);
        assert_eq!(app.list_state.selected(), Some(0));
        app.move_selection(2);
        assert_eq!(app.list_state.selected(), Some(2));
        app.move_selection(1);
        assert_eq!(app.list_state.selected(), Some(2));
    }

    #[test]
    fn test_move_selection_empty() {
        let _ = std::fs::remove_file(SAVE_FILE);
        let mut app = App::new();
        app.move_selection(1);
        assert!(app.filtered_indices.is_empty());
    }

    #[test]
    fn test_apply_search() {
        let mut app = App::new();
        app.todos.items = vec![
            make_todo("apple", Priority::Medium, false, 1),
            make_todo("banana", Priority::Medium, false, 2),
        ];
        app.input = "app".to_string();
        app.input_mode = InputMode::Searching;
        app.apply_search();

        assert_eq!(app.search_query, "app");
        assert_eq!(app.filtered_indices.len(), 1);
    }

    #[test]
    fn test_app_new() {
        let _ = std::fs::remove_file(SAVE_FILE);
        let app = App::new();
        assert!(app.todos.items.is_empty());
        assert!(app.should_quit == false);
        assert!(matches!(app.input_mode, InputMode::Normal));
        assert!(matches!(app.sort_mode, SortMode::Newest));
    }

    #[test]
    fn test_app_new_with_initial_item() {
        // We can't easily test with a save file in unit tests,
        // but we can verify the App struct initialization
        let app = App::new();
        assert!(app.input.is_empty());
        assert!(app.search_query.is_empty());
    }
}

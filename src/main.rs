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
};

const SAVE_FILE: &str = "todos.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Priority {
    High,
    Medium,
    Low,
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
}

struct App {
    todos: TodoList,
    list_state: ListState,
    input: String,
    input_mode: InputMode,
    search_query: String,
    filtered_indices: Vec<usize>,
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
        App {
            todos,
            list_state,
            input: String::new(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_indices,
            should_quit: false,
        }
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
            });
            self.refresh_filter();
            self.list_state.select(Some(self.filtered_indices.len().saturating_sub(1)));
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
        let new = (current as i32 + delta).clamp(0, self.filtered_indices.len() as i32 - 1) as usize;
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
            Span::styled("n", Style::new().fg(Color::Yellow)),
            Span::raw(":add "),
            Span::styled("e", Style::new().fg(Color::Yellow)),
            Span::raw(":edit "),
            Span::styled("d", Style::new().fg(Color::Yellow)),
            Span::raw(":del "),
            Span::styled("\u{2423}", Style::new().fg(Color::Yellow)),
            Span::raw(":done "),
            Span::styled("p", Style::new().fg(Color::Yellow)),
            Span::raw(":priority "),
            Span::styled("/", Style::new().fg(Color::Yellow)),
            Span::raw(":search "),
            Span::styled("q", Style::new().fg(Color::Yellow)),
            Span::raw(":quit"),
        ])
    } else {
        Line::from(vec![
            Span::styled("📋 Rust TODO", Style::new().bold().fg(Color::Cyan)),
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
            let priority_label =
                Span::styled(todo.priority.label(), Style::new().fg(todo.priority.color()));
            Line::from(vec![priority_label, Span::raw(" "), text])
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
                    format!(
                        "Selected: \"{}\" | {} | {}",
                        t.text,
                        t.priority.label(),
                        if t.done { "Done" } else { "Pending" }
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let footer =
                Paragraph::new(status).block(Block::bordered().title(" Status ").dim());
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
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kanban_tui::{storage::Storage, Board};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Application input mode
#[derive(Debug, PartialEq)]
enum InputMode {
    Normal,
    Creating,
    Editing,
}

/// Application state
struct App {
    board: Board,
    selected_column: usize,
    selected_task_index: Option<usize>,
    input_mode: InputMode,
    input_buffer: String,
    editing_task_id: Option<usize>,
    storage: Storage,
}

impl App {
    fn new() -> Self {
        let storage = Storage::new().expect("Failed to initialize storage");

        // Try to load existing board, or create new one
        let board = storage
            .load()
            .ok()
            .flatten()
            .unwrap_or_else(|| Board::new("My Kanban Board".to_string()));

        Self {
            board,
            selected_column: 0,
            selected_task_index: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_task_id: None,
            storage,
        }
    }

    /// Save the board to persistent storage
    fn save(&self) {
        if let Err(e) = self.storage.save(&self.board) {
            eprintln!("Failed to save board: {}", e);
        }
    }

    fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % self.board.columns.len();
        self.update_task_selection();
    }

    fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        } else {
            self.selected_column = self.board.columns.len() - 1;
        }
        self.update_task_selection();
    }

    fn update_task_selection(&mut self) {
        // Auto-select first task if column has tasks, otherwise clear selection
        let task_count = self.board.columns[self.selected_column].tasks.len();
        self.selected_task_index = if task_count > 0 { Some(0) } else { None };
    }

    fn next_task(&mut self) {
        let task_count = self.board.columns[self.selected_column].tasks.len();
        if task_count == 0 {
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(idx) => (idx + 1) % task_count,
            None => 0,
        });
    }

    fn previous_task(&mut self) {
        let task_count = self.board.columns[self.selected_column].tasks.len();
        if task_count == 0 {
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(idx) => {
                if idx > 0 {
                    idx - 1
                } else {
                    task_count - 1
                }
            }
            None => 0,
        });
    }

    fn delete_selected_task(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            // Get task ID before deletion
            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;

                // Remove the task
                self.board.columns[self.selected_column].remove_task(task_id);

                // Adjust selection after deletion
                let new_task_count = self.board.columns[self.selected_column].tasks.len();
                if new_task_count == 0 {
                    self.selected_task_index = None;
                } else if task_idx >= new_task_count {
                    // If we deleted the last task, select the new last task
                    self.selected_task_index = Some(new_task_count - 1);
                }
                // Otherwise keep the same index (which now points to the next task)

                // Save after deletion
                self.save();
            }
        }
    }

    fn move_task_left(&mut self) {
        // Can't move left from first column
        if self.selected_column == 0 {
            return;
        }

        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;
                let from_column = self.selected_column;
                let to_column = self.selected_column - 1;

                // Move the task
                if self.board.move_task(from_column, to_column, task_id).is_ok() {
                    // Update selected column
                    self.selected_column = to_column;

                    // Find the moved task in the new column and select it
                    let new_task_index = self.board.columns[to_column]
                        .tasks
                        .iter()
                        .position(|t| t.id == task_id);
                    self.selected_task_index = new_task_index;

                    // Save after move
                    self.save();
                }
            }
        }
    }

    fn move_task_right(&mut self) {
        // Can't move right from last column
        if self.selected_column >= self.board.columns.len() - 1 {
            return;
        }

        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;
                let from_column = self.selected_column;
                let to_column = self.selected_column + 1;

                // Move the task
                if self.board.move_task(from_column, to_column, task_id).is_ok() {
                    // Update selected column
                    self.selected_column = to_column;

                    // Find the moved task in the new column and select it
                    let new_task_index = self.board.columns[to_column]
                        .tasks
                        .iter()
                        .position(|t| t.id == task_id);
                    self.selected_task_index = new_task_index;

                    // Save after move
                    self.save();
                }
            }
        }
    }

    fn start_creating(&mut self) {
        self.input_mode = InputMode::Creating;
        self.input_buffer.clear();
    }

    fn create_task(&mut self) {
        if !self.input_buffer.is_empty() {
            let _ = self.board.add_task(self.selected_column, self.input_buffer.clone());
            self.input_buffer.clear();

            // Select the newly created task (last one in the column)
            let task_count = self.board.columns[self.selected_column].tasks.len();
            if task_count > 0 {
                self.selected_task_index = Some(task_count - 1);
            }

            // Save after creation
            self.save();
        }
        self.input_mode = InputMode::Normal;
    }

    fn cancel_creating(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    fn start_editing(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];
            if task_idx < column.tasks.len() {
                let task = &column.tasks[task_idx];
                self.editing_task_id = Some(task.id);
                self.input_buffer = task.title.clone();
                self.input_mode = InputMode::Editing;
            }
        }
    }

    fn save_edit(&mut self) {
        if let Some(task_id) = self.editing_task_id {
            if !self.input_buffer.is_empty() {
                let _ = self.board.update_task_title(
                    self.selected_column,
                    task_id,
                    self.input_buffer.clone(),
                );

                // Save after editing
                self.save();
            }
        }

        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    fn cancel_editing(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    fn handle_char_input(&mut self, c: char) {
        if self.input_mode == InputMode::Creating || self.input_mode == InputMode::Editing {
            self.input_buffer.push(c);
        }
    }

    fn handle_backspace(&mut self) {
        if self.input_mode == InputMode::Creating || self.input_mode == InputMode::Editing {
            self.input_buffer.pop();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run the application
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('n') => app.start_creating(),
                        KeyCode::Char('e') => app.start_editing(),
                        KeyCode::Char('h') | KeyCode::Left => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                app.move_task_left();
                            } else {
                                app.previous_column();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                app.move_task_right();
                            } else {
                                app.next_column();
                            }
                        }
                        KeyCode::Char('H') => app.move_task_left(),
                        KeyCode::Char('L') => app.move_task_right(),
                        KeyCode::Char('j') | KeyCode::Down => app.next_task(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous_task(),
                        KeyCode::Char('d') => app.delete_selected_task(),
                        _ => {}
                    },
                    InputMode::Creating => match key.code {
                        KeyCode::Enter => app.create_task(),
                        KeyCode::Esc => app.cancel_creating(),
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Allow Ctrl+C to quit
                                if c == 'c' {
                                    return Ok(());
                                }
                            } else {
                                app.handle_char_input(c);
                            }
                        }
                        KeyCode::Backspace => app.handle_backspace(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => app.save_edit(),
                        KeyCode::Esc => app.cancel_editing(),
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Allow Ctrl+C to quit
                                if c == 'c' {
                                    return Ok(());
                                }
                            } else {
                                app.handle_char_input(c);
                            }
                        }
                        KeyCode::Backspace => app.handle_backspace(),
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create main layout: columns area + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(size);

    // Render columns
    render_columns(f, app, chunks[0]);

    // Render status bar
    render_status_bar(f, app, chunks[1]);
}

fn render_columns(f: &mut Frame, app: &App, area: Rect) {
    let column_count = app.board.columns.len();
    let constraints = vec![Constraint::Percentage(100 / column_count as u16); column_count];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, column) in app.board.columns.iter().enumerate() {
        let is_selected_column = i == app.selected_column;
        let selected_task = if is_selected_column {
            app.selected_task_index
        } else {
            None
        };
        render_column(f, column, is_selected_column, selected_task, chunks[i]);
    }
}

fn render_column(
    f: &mut Frame,
    column: &kanban_tui::Column,
    is_selected_column: bool,
    selected_task_index: Option<usize>,
    area: Rect,
) {
    let color = if is_selected_column {
        Color::Cyan
    } else {
        Color::White
    };

    let border_style = if is_selected_column {
        Style::default()
            .fg(color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    let title = if is_selected_column {
        format!("▶ {} ({}) ◀", column.name, column.tasks.len())
    } else {
        format!("{} ({})", column.name, column.tasks.len())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Create list items from tasks with numbering and selection highlighting
    let items: Vec<ListItem> = column
        .tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let content = format!("{}. {}", idx + 1, task.title);
            let is_selected_task = selected_task_index == Some(idx);

            let style = if is_selected_task {
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (text, style) = match app.input_mode {
        InputMode::Normal => {
            let help = vec![
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": new | "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": edit | "),
                Span::styled("h/l", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": columns | "),
                Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": tasks | "),
                Span::styled("Shift+h/l", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": move | "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": delete | "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": quit"),
            ];
            (Line::from(help), Style::default().fg(Color::Gray))
        }
        InputMode::Creating => {
            let prompt = vec![
                Span::styled(
                    "Creating task: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&app.input_buffer),
                Span::styled("█", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to save | "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ];
            (
                Line::from(prompt),
                Style::default().fg(Color::Yellow),
            )
        }
        InputMode::Editing => {
            let prompt = vec![
                Span::styled(
                    "Editing task: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&app.input_buffer),
                Span::styled("█", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to save | "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ];
            (
                Line::from(prompt),
                Style::default().fg(Color::Green),
            )
        }
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper function to create App with temporary storage for testing
    fn test_app() -> App {
        let temp_dir = env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_file = temp_dir.join(format!("kanban-test-app-{}.json", timestamp));
        let storage = Storage::with_path(test_file);
        let board = Board::new("My Kanban Board".to_string());

        App {
            board,
            selected_column: 0,
            selected_task_index: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_task_id: None,
            storage,
        }
    }

    #[test]
    fn test_app_initialization() {
        let app = test_app();
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_task_index, None);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.board.columns.len(), 3);
    }

    #[test]
    fn test_next_column_navigation() {
        let mut app = test_app();

        // Start at column 0
        assert_eq!(app.selected_column, 0);

        // Move to column 1
        app.next_column();
        assert_eq!(app.selected_column, 1);

        // Move to column 2
        app.next_column();
        assert_eq!(app.selected_column, 2);

        // Wrap back to column 0
        app.next_column();
        assert_eq!(app.selected_column, 0);
    }

    #[test]
    fn test_previous_column_navigation() {
        let mut app = test_app();

        // Start at column 0, go backwards (should wrap to last column)
        assert_eq!(app.selected_column, 0);
        app.previous_column();
        assert_eq!(app.selected_column, 2);

        // Move back to column 1
        app.previous_column();
        assert_eq!(app.selected_column, 1);

        // Move to column 0
        app.previous_column();
        assert_eq!(app.selected_column, 0);
    }

    #[test]
    fn test_start_creating_task() {
        let mut app = test_app();

        // Add some text to input buffer to verify it gets cleared
        app.input_buffer = "old text".to_string();

        app.start_creating();

        assert_eq!(app.input_mode, InputMode::Creating);
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_create_task_with_input() {
        let mut app = test_app();

        // Set up creating mode with input
        app.start_creating();
        app.input_buffer = "My new task".to_string();

        // Get initial task count
        let initial_count = app.board.columns[0].tasks.len();

        // Create the task
        app.create_task();

        // Verify task was added
        assert_eq!(app.board.columns[0].tasks.len(), initial_count + 1);
        assert_eq!(
            app.board.columns[0].tasks[initial_count].title,
            "My new task"
        );

        // Verify state reset
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_create_task_with_empty_input() {
        let mut app = test_app();

        // Set up creating mode with empty input
        app.start_creating();
        assert_eq!(app.input_buffer, "");

        let initial_count = app.board.columns[0].tasks.len();

        // Try to create with empty buffer
        app.create_task();

        // No task should be added
        assert_eq!(app.board.columns[0].tasks.len(), initial_count);

        // But mode should still switch back to Normal
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_create_task_in_different_columns() {
        let mut app = test_app();

        // Create task in column 0
        app.selected_column = 0;
        app.start_creating();
        app.input_buffer = "Task in column 0".to_string();
        app.create_task();
        assert_eq!(app.board.columns[0].tasks.len(), 1);

        // Create task in column 1
        app.selected_column = 1;
        app.start_creating();
        app.input_buffer = "Task in column 1".to_string();
        app.create_task();
        assert_eq!(app.board.columns[1].tasks.len(), 1);

        // Create task in column 2
        app.selected_column = 2;
        app.start_creating();
        app.input_buffer = "Task in column 2".to_string();
        app.create_task();
        assert_eq!(app.board.columns[2].tasks.len(), 1);

        // Verify tasks are in correct columns
        assert_eq!(app.board.columns[0].tasks[0].title, "Task in column 0");
        assert_eq!(app.board.columns[1].tasks[0].title, "Task in column 1");
        assert_eq!(app.board.columns[2].tasks[0].title, "Task in column 2");
    }

    #[test]
    fn test_cancel_creating() {
        let mut app = test_app();

        // Start creating and add some input
        app.start_creating();
        app.input_buffer = "Some text".to_string();

        // Cancel
        app.cancel_creating();

        // Verify state
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_handle_char_input_in_creating_mode() {
        let mut app = test_app();

        app.start_creating();

        app.handle_char_input('H');
        app.handle_char_input('e');
        app.handle_char_input('l');
        app.handle_char_input('l');
        app.handle_char_input('o');

        assert_eq!(app.input_buffer, "Hello");
    }

    #[test]
    fn test_handle_char_input_in_normal_mode() {
        let mut app = test_app();

        // Try to input while in Normal mode
        assert_eq!(app.input_mode, InputMode::Normal);

        app.handle_char_input('H');
        app.handle_char_input('i');

        // Buffer should remain empty
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_handle_backspace_in_creating_mode() {
        let mut app = test_app();

        app.start_creating();
        app.input_buffer = "Hello World".to_string();

        // Remove 'd'
        app.handle_backspace();
        assert_eq!(app.input_buffer, "Hello Worl");

        // Remove 'l'
        app.handle_backspace();
        assert_eq!(app.input_buffer, "Hello Wor");

        // Remove all remaining characters
        for _ in 0..9 {
            app.handle_backspace();
        }
        assert_eq!(app.input_buffer, "");

        // Backspace on empty buffer should not panic
        app.handle_backspace();
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_handle_backspace_in_normal_mode() {
        let mut app = test_app();

        // Set buffer manually and stay in Normal mode
        app.input_buffer = "Test".to_string();
        assert_eq!(app.input_mode, InputMode::Normal);

        // Backspace should not affect buffer in Normal mode
        app.handle_backspace();
        assert_eq!(app.input_buffer, "Test");
    }

    #[test]
    fn test_complete_task_creation_workflow() {
        let mut app = test_app();

        // Navigate to column 1
        app.next_column();
        assert_eq!(app.selected_column, 1);

        // Start creating
        app.start_creating();
        assert_eq!(app.input_mode, InputMode::Creating);

        // Type task title
        for c in "Fix the bug".chars() {
            app.handle_char_input(c);
        }
        assert_eq!(app.input_buffer, "Fix the bug");

        // Create the task
        app.create_task();

        // Verify
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks[0].title, "Fix the bug");
    }

    #[test]
    fn test_task_selection_auto_updates_on_column_change() {
        let mut app = test_app();

        // Add tasks to columns
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(1, "Task 2".to_string()).unwrap();

        // Initially on column 0 with no selection
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_task_index, None);

        // Navigate to column 0 (which has tasks)
        app.next_column();
        app.previous_column();
        // Should auto-select first task
        assert_eq!(app.selected_task_index, Some(0));

        // Navigate to column 1 (which has tasks)
        app.next_column();
        // Should auto-select first task of new column
        assert_eq!(app.selected_task_index, Some(0));

        // Navigate to column 2 (which has no tasks)
        app.next_column();
        // Should clear selection
        assert_eq!(app.selected_task_index, None);
    }

    #[test]
    fn test_next_task_navigation() {
        let mut app = test_app();

        // Add 3 tasks to column 0
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(0, "Task 2".to_string()).unwrap();
        app.board.add_task(0, "Task 3".to_string()).unwrap();

        // Start with no selection
        assert_eq!(app.selected_task_index, None);

        // First next_task should select task 0
        app.next_task();
        assert_eq!(app.selected_task_index, Some(0));

        // Move to task 1
        app.next_task();
        assert_eq!(app.selected_task_index, Some(1));

        // Move to task 2
        app.next_task();
        assert_eq!(app.selected_task_index, Some(2));

        // Wrap back to task 0
        app.next_task();
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[test]
    fn test_previous_task_navigation() {
        let mut app = test_app();

        // Add 3 tasks to column 0
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(0, "Task 2".to_string()).unwrap();
        app.board.add_task(0, "Task 3".to_string()).unwrap();

        // Start with no selection
        assert_eq!(app.selected_task_index, None);

        // First previous_task should select task 0
        app.previous_task();
        assert_eq!(app.selected_task_index, Some(0));

        // Going backwards should wrap to last task
        app.previous_task();
        assert_eq!(app.selected_task_index, Some(2));

        // Move to task 1
        app.previous_task();
        assert_eq!(app.selected_task_index, Some(1));

        // Move to task 0
        app.previous_task();
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[test]
    fn test_task_navigation_on_empty_column() {
        let mut app = test_app();

        // Column 0 is empty
        assert_eq!(app.board.columns[0].tasks.len(), 0);

        // next_task on empty column should do nothing
        app.next_task();
        assert_eq!(app.selected_task_index, None);

        // previous_task on empty column should do nothing
        app.previous_task();
        assert_eq!(app.selected_task_index, None);
    }

    #[test]
    fn test_delete_selected_task() {
        let mut app = test_app();

        // Add 3 tasks
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(0, "Task 2".to_string()).unwrap();
        app.board.add_task(0, "Task 3".to_string()).unwrap();

        // Select first task
        app.selected_task_index = Some(0);

        // Delete it
        app.delete_selected_task();

        // Should have 2 tasks remaining
        assert_eq!(app.board.columns[0].tasks.len(), 2);
        // Task 2 is now at index 0
        assert_eq!(app.board.columns[0].tasks[0].title, "Task 2");
        // Selection should still be at index 0 (pointing to what was Task 2)
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[test]
    fn test_delete_last_task_in_list() {
        let mut app = test_app();

        // Add 3 tasks
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(0, "Task 2".to_string()).unwrap();
        app.board.add_task(0, "Task 3".to_string()).unwrap();

        // Select last task (index 2)
        app.selected_task_index = Some(2);

        // Delete it
        app.delete_selected_task();

        // Should have 2 tasks remaining
        assert_eq!(app.board.columns[0].tasks.len(), 2);
        // Selection should move to new last task (index 1)
        assert_eq!(app.selected_task_index, Some(1));
        assert_eq!(app.board.columns[0].tasks[1].title, "Task 2");
    }

    #[test]
    fn test_delete_only_task() {
        let mut app = test_app();

        // Add one task
        app.board.add_task(0, "Only task".to_string()).unwrap();

        // Select it
        app.selected_task_index = Some(0);

        // Delete it
        app.delete_selected_task();

        // Should have no tasks
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        // Selection should be cleared
        assert_eq!(app.selected_task_index, None);
    }

    #[test]
    fn test_delete_with_no_selection() {
        let mut app = test_app();

        // Add task
        app.board.add_task(0, "Task 1".to_string()).unwrap();

        // No selection
        assert_eq!(app.selected_task_index, None);

        // Try to delete - should do nothing
        app.delete_selected_task();

        // Task should still exist
        assert_eq!(app.board.columns[0].tasks.len(), 1);
    }

    #[test]
    fn test_delete_middle_task() {
        let mut app = test_app();

        // Add 3 tasks
        app.board.add_task(0, "Task 1".to_string()).unwrap();
        app.board.add_task(0, "Task 2".to_string()).unwrap();
        app.board.add_task(0, "Task 3".to_string()).unwrap();

        // Select middle task
        app.selected_task_index = Some(1);

        // Delete it
        app.delete_selected_task();

        // Should have 2 tasks
        assert_eq!(app.board.columns[0].tasks.len(), 2);
        assert_eq!(app.board.columns[0].tasks[0].title, "Task 1");
        assert_eq!(app.board.columns[0].tasks[1].title, "Task 3");
        // Selection should stay at index 1 (now pointing to Task 3)
        assert_eq!(app.selected_task_index, Some(1));
    }

    #[test]
    fn test_create_task_selects_new_task() {
        let mut app = test_app();

        // Create a task
        app.start_creating();
        app.input_buffer = "New task".to_string();
        app.create_task();

        // Should select the newly created task
        assert_eq!(app.selected_task_index, Some(0));
        assert_eq!(app.board.columns[0].tasks[0].title, "New task");

        // Create another task
        app.start_creating();
        app.input_buffer = "Another task".to_string();
        app.create_task();

        // Should select the newest task
        assert_eq!(app.selected_task_index, Some(1));
    }

    #[test]
    fn test_complete_deletion_workflow() {
        let mut app = test_app();

        // Create 3 tasks
        for i in 1..=3 {
            app.start_creating();
            app.input_buffer = format!("Task {}", i);
            app.create_task();
        }

        assert_eq!(app.board.columns[0].tasks.len(), 3);
        assert_eq!(app.selected_task_index, Some(2)); // Last created

        // Navigate to first task
        app.previous_task();
        app.previous_task();
        assert_eq!(app.selected_task_index, Some(0));

        // Delete first task
        app.delete_selected_task();
        assert_eq!(app.board.columns[0].tasks.len(), 2);
        assert_eq!(app.board.columns[0].tasks[0].title, "Task 2");

        // Delete current task (now Task 2)
        app.delete_selected_task();
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.board.columns[0].tasks[0].title, "Task 3");

        // Delete last task
        app.delete_selected_task();
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.selected_task_index, None);
    }

    #[test]
    fn test_move_task_right() {
        let mut app = test_app();

        // Add task to column 0
        let task_id = app.board.add_task(0, "My task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = Some(0);

        // Move task to column 1
        app.move_task_right();

        // Verify task moved
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks[0].title, "My task");
        assert_eq!(app.board.columns[1].tasks[0].id, task_id);

        // Verify selection followed task
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[test]
    fn test_move_task_left() {
        let mut app = test_app();

        // Add task to column 1
        let task_id = app.board.add_task(1, "My task".to_string()).unwrap();
        app.selected_column = 1;
        app.selected_task_index = Some(0);

        // Move task to column 0
        app.move_task_left();

        // Verify task moved
        assert_eq!(app.board.columns[1].tasks.len(), 0);
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.board.columns[0].tasks[0].title, "My task");
        assert_eq!(app.board.columns[0].tasks[0].id, task_id);

        // Verify selection followed task
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[test]
    fn test_move_task_cannot_move_left_from_first_column() {
        let mut app = test_app();

        // Add task to column 0
        app.board.add_task(0, "Task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = Some(0);

        // Try to move left from first column
        app.move_task_left();

        // Task should still be in column 0
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.selected_column, 0);
    }

    #[test]
    fn test_move_task_cannot_move_right_from_last_column() {
        let mut app = test_app();

        // Add task to last column (column 2)
        app.board.add_task(2, "Task".to_string()).unwrap();
        app.selected_column = 2;
        app.selected_task_index = Some(0);

        // Try to move right from last column
        app.move_task_right();

        // Task should still be in column 2
        assert_eq!(app.board.columns[2].tasks.len(), 1);
        assert_eq!(app.selected_column, 2);
    }

    #[test]
    fn test_move_task_with_no_selection() {
        let mut app = test_app();

        // Add task but don't select it
        app.board.add_task(0, "Task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = None;

        // Try to move
        app.move_task_right();

        // Task should still be in column 0
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks.len(), 0);
    }

    #[test]
    fn test_move_task_through_all_columns() {
        let mut app = test_app();

        // Add task to column 0
        let task_id = app.board.add_task(0, "Traveling task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = Some(0);

        // Move from column 0 to 1
        app.move_task_right();
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks[0].id, task_id);

        // Move from column 1 to 2
        app.move_task_right();
        assert_eq!(app.selected_column, 2);
        assert_eq!(app.board.columns[1].tasks.len(), 0);
        assert_eq!(app.board.columns[2].tasks.len(), 1);
        assert_eq!(app.board.columns[2].tasks[0].id, task_id);

        // Move from column 2 to 1
        app.move_task_left();
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.board.columns[2].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks[0].id, task_id);

        // Move from column 1 to 0
        app.move_task_left();
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.board.columns[1].tasks.len(), 0);
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.board.columns[0].tasks[0].id, task_id);
    }

    #[test]
    fn test_move_task_to_column_with_existing_tasks() {
        let mut app = test_app();

        // Add multiple tasks to column 1
        app.board.add_task(1, "Existing 1".to_string()).unwrap();
        app.board.add_task(1, "Existing 2".to_string()).unwrap();

        // Add task to column 0 and select it
        let task_id = app.board.add_task(0, "Moving task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = Some(0);

        // Move to column 1 (which already has tasks)
        app.move_task_right();

        // Verify task was added to column 1
        assert_eq!(app.board.columns[1].tasks.len(), 3);
        assert_eq!(app.board.columns[1].tasks[2].title, "Moving task");
        assert_eq!(app.board.columns[1].tasks[2].id, task_id);

        // Verify selection
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.selected_task_index, Some(2)); // Should be at end
    }

    #[test]
    fn test_complete_kanban_workflow() {
        let mut app = test_app();

        // Create a task in "To Do" column (column 0)
        app.selected_column = 0;
        app.start_creating();
        app.input_buffer = "Implement feature".to_string();
        app.create_task();

        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.selected_task_index, Some(0));

        // Move to "In Progress" (column 1)
        app.move_task_right();
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.board.columns[1].tasks[0].title, "Implement feature");

        // Move to "Done" (column 2)
        app.move_task_right();
        assert_eq!(app.selected_column, 2);
        assert_eq!(app.board.columns[1].tasks.len(), 0);
        assert_eq!(app.board.columns[2].tasks.len(), 1);
        assert_eq!(app.board.columns[2].tasks[0].title, "Implement feature");

        // Task is complete!
        assert_eq!(app.board.columns[2].tasks[0].title, "Implement feature");
    }

    #[test]
    fn test_storage_persistence() {
        let temp_dir = env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_file = temp_dir.join(format!("kanban-test-persist-{}.json", timestamp));
        let storage = Storage::with_path(test_file.clone());

        // Create board and add tasks
        let mut board = Board::new("Test Board".to_string());
        board.add_task(0, "Task 1".to_string()).unwrap();
        board.add_task(1, "Task 2".to_string()).unwrap();

        // Save to storage
        storage.save(&board).unwrap();

        // Load from storage
        let loaded = storage.load().unwrap();
        assert!(loaded.is_some());
        let loaded_board = loaded.unwrap();

        // Verify
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns[0].tasks.len(), 1);
        assert_eq!(loaded_board.columns[0].tasks[0].title, "Task 1");
        assert_eq!(loaded_board.columns[1].tasks.len(), 1);
        assert_eq!(loaded_board.columns[1].tasks[0].title, "Task 2");

        // Cleanup
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_auto_save_on_create() {
        let mut app = test_app();
        let storage_path = app.storage.file_path().clone();

        // Create a task
        app.start_creating();
        app.input_buffer = "Auto-saved task".to_string();
        app.create_task();

        // Load from storage to verify it was saved
        let loaded = app.storage.load().unwrap().unwrap();
        assert_eq!(loaded.columns[0].tasks.len(), 1);
        assert_eq!(loaded.columns[0].tasks[0].title, "Auto-saved task");

        // Cleanup
        std::fs::remove_file(storage_path).ok();
    }

    #[test]
    fn test_auto_save_on_delete() {
        let mut app = test_app();
        let storage_path = app.storage.file_path().clone();

        // Create and then delete a task
        app.board.add_task(0, "To be deleted".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.delete_selected_task();

        // Verify saved state
        let loaded = app.storage.load().unwrap().unwrap();
        assert_eq!(loaded.columns[0].tasks.len(), 0);

        // Cleanup
        std::fs::remove_file(storage_path).ok();
    }

    #[test]
    fn test_auto_save_on_move() {
        let mut app = test_app();
        let storage_path = app.storage.file_path().clone();

        // Create task and move it
        app.board.add_task(0, "Moving task".to_string()).unwrap();
        app.selected_column = 0;
        app.selected_task_index = Some(0);
        app.move_task_right();

        // Verify saved state
        let loaded = app.storage.load().unwrap().unwrap();
        assert_eq!(loaded.columns[0].tasks.len(), 0);
        assert_eq!(loaded.columns[1].tasks.len(), 1);
        assert_eq!(loaded.columns[1].tasks[0].title, "Moving task");

        // Cleanup
        std::fs::remove_file(storage_path).ok();
    }

    #[test]
    fn test_start_editing() {
        let mut app = test_app();

        // Create a task
        app.board.add_task(0, "Original Title".to_string()).unwrap();
        app.selected_task_index = Some(0);

        // Start editing
        app.start_editing();

        // Verify we're in editing mode
        assert_eq!(app.input_mode, InputMode::Editing);
        // Buffer should be pre-populated with task title
        assert_eq!(app.input_buffer, "Original Title");
        // Should track which task is being edited
        assert!(app.editing_task_id.is_some());
    }

    #[test]
    fn test_save_edit() {
        let mut app = test_app();

        // Create a task and start editing it
        let task_id = app.board.add_task(0, "Original Title".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.start_editing();

        // Modify the title
        app.input_buffer = "Updated Title".to_string();

        // Save the edit
        app.save_edit();

        // Verify changes
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.editing_task_id, None);
        assert_eq!(app.board.columns[0].tasks[0].title, "Updated Title");
        assert_eq!(app.board.columns[0].tasks[0].id, task_id);
    }

    #[test]
    fn test_cancel_editing() {
        let mut app = test_app();

        // Create a task and start editing it
        app.board.add_task(0, "Original Title".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.start_editing();

        // Modify the buffer
        app.input_buffer = "Changed but cancelled".to_string();

        // Cancel editing
        app.cancel_editing();

        // Verify state was reset
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.editing_task_id, None);
        // Original title should be unchanged
        assert_eq!(app.board.columns[0].tasks[0].title, "Original Title");
    }

    #[test]
    fn test_edit_with_no_selection() {
        let mut app = test_app();

        // Create a task but don't select it
        app.board.add_task(0, "Task".to_string()).unwrap();
        app.selected_task_index = None;

        // Try to edit - should do nothing
        app.start_editing();

        // Should still be in Normal mode
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.editing_task_id, None);
    }

    #[test]
    fn test_save_edit_with_empty_buffer() {
        let mut app = test_app();

        // Create a task and start editing it
        app.board.add_task(0, "Original Title".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.start_editing();

        // Clear the buffer
        app.input_buffer.clear();

        // Try to save - should not update title
        app.save_edit();

        // Should return to Normal mode
        assert_eq!(app.input_mode, InputMode::Normal);
        // Title should remain unchanged
        assert_eq!(app.board.columns[0].tasks[0].title, "Original Title");
    }

    #[test]
    fn test_complete_edit_workflow() {
        let mut app = test_app();

        // Create a task
        app.start_creating();
        app.input_buffer = "Initial Task".to_string();
        app.create_task();

        assert_eq!(app.board.columns[0].tasks[0].title, "Initial Task");
        assert_eq!(app.selected_task_index, Some(0));

        // Edit the task
        app.start_editing();
        assert_eq!(app.input_mode, InputMode::Editing);
        assert_eq!(app.input_buffer, "Initial Task");

        // Modify the title
        app.input_buffer.clear();
        for c in "Updated Task".chars() {
            app.handle_char_input(c);
        }
        assert_eq!(app.input_buffer, "Updated Task");

        // Save the edit
        app.save_edit();

        // Verify the complete workflow
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.board.columns[0].tasks[0].title, "Updated Task");
    }

    #[test]
    fn test_handle_char_input_in_editing_mode() {
        let mut app = test_app();

        // Create a task and start editing
        app.board.add_task(0, "Test".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.start_editing();

        // Clear buffer and add new text
        app.input_buffer.clear();

        app.handle_char_input('N');
        app.handle_char_input('e');
        app.handle_char_input('w');

        assert_eq!(app.input_buffer, "New");
    }

    #[test]
    fn test_auto_save_on_edit() {
        let mut app = test_app();
        let storage_path = app.storage.file_path().clone();

        // Create and edit a task
        app.board.add_task(0, "Original".to_string()).unwrap();
        app.selected_task_index = Some(0);
        app.start_editing();
        app.input_buffer = "Edited".to_string();
        app.save_edit();

        // Verify saved state
        let loaded = app.storage.load().unwrap().unwrap();
        assert_eq!(loaded.columns[0].tasks.len(), 1);
        assert_eq!(loaded.columns[0].tasks[0].title, "Edited");

        // Cleanup
        std::fs::remove_file(storage_path).ok();
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kanban_tui::Board;
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
}

/// Application state
struct App {
    board: Board,
    selected_column: usize,
    input_mode: InputMode,
    input_buffer: String,
}

impl App {
    fn new() -> Self {
        Self {
            board: Board::new("My Kanban Board".to_string()),
            selected_column: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }

    fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % self.board.columns.len();
    }

    fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        } else {
            self.selected_column = self.board.columns.len() - 1;
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
        }
        self.input_mode = InputMode::Normal;
    }

    fn cancel_creating(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    fn handle_char_input(&mut self, c: char) {
        if self.input_mode == InputMode::Creating {
            self.input_buffer.push(c);
        }
    }

    fn handle_backspace(&mut self) {
        if self.input_mode == InputMode::Creating {
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
                        KeyCode::Char('h') | KeyCode::Left => app.previous_column(),
                        KeyCode::Char('l') | KeyCode::Right => app.next_column(),
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
        let is_selected = i == app.selected_column;
        render_column(f, column, is_selected, chunks[i]);
    }
}

fn render_column(f: &mut Frame, column: &kanban_tui::Column, is_selected: bool, area: Rect) {
    let color = if is_selected {
        Color::Cyan
    } else {
        Color::White
    };

    let border_style = if is_selected {
        Style::default()
            .fg(color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    let title = if is_selected {
        format!("▶ {} ({}) ◀", column.name, column.tasks.len())
    } else {
        format!("{} ({})", column.name, column.tasks.len())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Create list items from tasks
    let items: Vec<ListItem> = column
        .tasks
        .iter()
        .map(|task| {
            let content = format!("• {}", task.title);
            ListItem::new(content).style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (text, style) = match app.input_mode {
        InputMode::Normal => {
            let help = vec![
                Span::raw("Press "),
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to create task | "),
                Span::styled("h/l", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" or "),
                Span::styled("←/→", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to navigate | "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
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

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.board.columns.len(), 3);
    }

    #[test]
    fn test_next_column_navigation() {
        let mut app = App::new();

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
        let mut app = App::new();

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
        let mut app = App::new();

        // Add some text to input buffer to verify it gets cleared
        app.input_buffer = "old text".to_string();

        app.start_creating();

        assert_eq!(app.input_mode, InputMode::Creating);
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_create_task_with_input() {
        let mut app = App::new();

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
        let mut app = App::new();

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
        let mut app = App::new();

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
        let mut app = App::new();

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
        let mut app = App::new();

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
        let mut app = App::new();

        // Try to input while in Normal mode
        assert_eq!(app.input_mode, InputMode::Normal);

        app.handle_char_input('H');
        app.handle_char_input('i');

        // Buffer should remain empty
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_handle_backspace_in_creating_mode() {
        let mut app = App::new();

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
        let mut app = App::new();

        // Set buffer manually and stay in Normal mode
        app.input_buffer = "Test".to_string();
        assert_eq!(app.input_mode, InputMode::Normal);

        // Backspace should not affect buffer in Normal mode
        app.handle_backspace();
        assert_eq!(app.input_buffer, "Test");
    }

    #[test]
    fn test_complete_task_creation_workflow() {
        let mut app = App::new();

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
}

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
#[derive(PartialEq)]
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

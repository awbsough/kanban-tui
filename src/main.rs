use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let res = run_app(&mut terminal);

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
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Create three vertical columns for Kanban board
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(size);

            // Column 1: To Do
            let todo_block = Block::default()
                .title("To Do")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan));
            let todo_text = Paragraph::new(vec![
                Line::from(vec![Span::raw("Welcome to Kanban TUI!")]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    "Press 'q' to quit",
                    Style::default().add_modifier(Modifier::ITALIC),
                )]),
            ])
            .block(todo_block);
            f.render_widget(todo_text, chunks[0]);

            // Column 2: In Progress
            let progress_block = Block::default()
                .title("In Progress")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));
            let progress_text = Paragraph::new("").block(progress_block);
            f.render_widget(progress_text, chunks[1]);

            // Column 3: Done
            let done_block = Block::default()
                .title("Done")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Green));
            let done_text = Paragraph::new("").block(done_block);
            f.render_widget(done_text, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}

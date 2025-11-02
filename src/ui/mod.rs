//! UI rendering modules for the Kanban TUI.

mod board_selector;
mod column;
mod status_bar;
mod task_detail;

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub use board_selector::render_board_selector;
pub use column::render_column;
pub use status_bar::render_status_bar;
pub use task_detail::render_task_detail;

/// Main UI rendering function
pub fn ui(f: &mut Frame, app: &App) {
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

    // Render task detail popup if in viewing mode
    if app.input_mode == InputMode::Viewing {
        render_task_detail(f, app, size);
    }

    // Render board selector if in board selection mode
    if app.input_mode == InputMode::SelectingBoard {
        render_board_selector(f, app, size);
    }
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

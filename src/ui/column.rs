//! Column rendering for the Kanban TUI.

use kanban_tui::{Column, Priority};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_column(
    f: &mut Frame,
    column: &Column,
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
            // Build task display with priority and tags
            let priority_symbol = task.priority.symbol();
            let priority_str = if !priority_symbol.is_empty() {
                format!("{} ", priority_symbol)
            } else {
                String::new()
            };

            let tag_indicator = if !task.tags.is_empty() {
                format!(" [tags: {}]", task.tags.len())
            } else {
                String::new()
            };

            let content = format!("{}. {}{}{}", idx + 1, priority_str, task.title, tag_indicator);
            let is_selected_task = selected_task_index == Some(idx);

            // Determine color based on priority
            let priority_color = match task.priority {
                Priority::High => Color::Red,
                Priority::Medium => Color::Yellow,
                Priority::Low => Color::Green,
                Priority::None => Color::White,
            };

            let style = if is_selected_task {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(priority_color)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

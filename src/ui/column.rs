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
    // Calculate card width based on available area (accounting for borders and padding)
    let card_width = (area.width.saturating_sub(4)).max(20) as usize;

    let items: Vec<ListItem> = column
        .tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            use ratatui::text::{Line, Span};

            let is_selected_task = selected_task_index == Some(idx);

            // Determine color based on priority
            let priority_color = match task.priority {
                Priority::High => Color::Red,
                Priority::Medium => Color::Yellow,
                Priority::Low => Color::Green,
                Priority::None => Color::White,
            };

            // Base style for the card
            let base_style = if is_selected_task {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(priority_color)
            };

            let border_style = if is_selected_task {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
            } else {
                Style::default().fg(priority_color)
            };

            let meta_style = if is_selected_task {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Build card content lines (text content only, for padding calculation)
            let mut content_lines = Vec::new();

            // Line 1: Number, priority symbol, and title
            let priority_symbol = task.priority.symbol();
            let priority_str = if !priority_symbol.is_empty() {
                format!("{} ", priority_symbol)
            } else {
                String::new()
            };
            let title_line = format!("{}. {}{}", idx + 1, priority_str, task.title);
            content_lines.push(title_line);

            // Line 2: Tags (if present)
            if !task.tags.is_empty() {
                content_lines.push(format!("  {}", task.tags.join(", ")));
            }

            // Line 3: Due date (if present)
            if let Some(due) = &task.due_date {
                content_lines.push(format!("  due: {}", due));
            }

            // Build the bordered card
            let mut lines = Vec::new();

            // Top border: ╭──────╮
            lines.push(Line::from(vec![
                Span::styled(
                    format!("╭{}╮", "─".repeat(card_width.saturating_sub(2))),
                    border_style
                )
            ]));

            // Content lines with side borders: │ content │
            for content in &content_lines {
                let display_content = if content.len() > card_width.saturating_sub(4) {
                    // Truncate if too long
                    format!("{:width$}", &content[..card_width.saturating_sub(7)], width = card_width.saturating_sub(4))
                } else {
                    // Pad to fill width
                    format!("{:width$}", content, width = card_width.saturating_sub(4))
                };

                let line_style = if content == &content_lines[0] {
                    base_style // First line uses base style (title)
                } else {
                    meta_style // Metadata lines use meta style
                };

                lines.push(Line::from(vec![
                    Span::styled("│ ", border_style),
                    Span::styled(display_content, line_style),
                    Span::styled(" │", border_style),
                ]));
            }

            // Bottom border: ╰──────╯
            lines.push(Line::from(vec![
                Span::styled(
                    format!("╰{}╯", "─".repeat(card_width.saturating_sub(2))),
                    border_style
                )
            ]));

            // Add empty line for spacing between cards
            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

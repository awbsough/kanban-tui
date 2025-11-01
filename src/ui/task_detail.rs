//! Task detail popup rendering for the Kanban TUI.

use crate::app::App;
use kanban_tui::Priority;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_task_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(task_idx) = app.selected_task_index {
        let column = &app.board.columns[app.selected_column];
        if task_idx < column.tasks.len() {
            let task = &column.tasks[task_idx];

            // Create centered popup area
            let popup_width = 60.min(area.width - 4);
            let popup_height = 20.min(area.height - 4);
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;

            let popup_area = Rect {
                x: area.x + popup_x,
                y: area.y + popup_y,
                width: popup_width,
                height: popup_height,
            };

            // Build content lines
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&task.title),
                ]),
                Line::from(""),
            ];

            // Description
            if let Some(desc) = &task.description {
                if !desc.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "Description: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(desc.as_str()));
                    lines.push(Line::from(""));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("(none)", Style::default().fg(Color::Gray)),
                ]));
                lines.push(Line::from(""));
            }

            // Priority with color coding
            let priority_color = match task.priority {
                Priority::High => Color::Red,
                Priority::Medium => Color::Yellow,
                Priority::Low => Color::Green,
                Priority::None => Color::Gray,
            };
            lines.push(Line::from(vec![
                Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} {}", task.priority.symbol(), task.priority),
                    Style::default()
                        .fg(priority_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            // Tags with color coding
            if !task.tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(task.tags.join(", "), Style::default().fg(Color::Cyan)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("(none)", Style::default().fg(Color::Gray)),
                ]));
            }
            lines.push(Line::from(""));

            // Timestamps
            lines.push(Line::from(vec![
                Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.created_at),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.updated_at),
            ]));

            // Due date
            if let Some(due) = &task.due_date {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(due),
                ]));
            }

            // Clear the area and render popup
            f.render_widget(Clear, popup_area);
            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Task Details (press Esc to close) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, popup_area);
        }
    }
}

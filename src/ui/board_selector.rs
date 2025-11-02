//! Board selector popup rendering for the Kanban TUI.

use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_board_selector(f: &mut Frame, app: &App, area: Rect) {
    // Create centered popup area
    let popup_width = 50.min(area.width - 4);
    let popup_height = (app.available_boards.len() as u16 + 6).min(area.height - 4);
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Build board list items
    let items: Vec<ListItem> = app
        .available_boards
        .iter()
        .enumerate()
        .map(|(idx, board_name)| {
            let is_selected = app.selected_board_index == Some(idx);
            let is_current = board_name == &app.current_board_name;

            let prefix = if is_current { "✓ " } else { "  " };
            let content = format!("{}{}", prefix, board_name);

            let style = if is_selected {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    // Clear the area and render popup
    f.render_widget(Clear, popup_area);

    let list = List::new(items).block(
        Block::default()
            .title(" Select Board ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    // Split area for list and help text
    let list_height = popup_height.saturating_sub(4);
    let list_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_width,
        height: list_height,
    };

    let help_area = Rect {
        x: popup_area.x,
        y: popup_area.y + list_height,
        width: popup_width,
        height: 4,
    };

    f.render_widget(list, list_area);

    // Render help text at bottom
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": switch | "),
            Span::styled("n/B", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": new | "),
            Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": delete | "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": cancel"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(help, help_area);
}

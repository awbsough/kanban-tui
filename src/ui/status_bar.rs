//! Status bar rendering for the Kanban TUI.

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (text, style) = match app.input_mode {
        InputMode::Normal => (build_normal_mode_help(), Style::default().fg(Color::Gray)),
        InputMode::Creating => (
            build_input_prompt("Creating task: ", &app.input_buffer),
            Style::default().fg(Color::Yellow),
        ),
        InputMode::Editing => (
            build_input_prompt("Editing title: ", &app.input_buffer),
            Style::default().fg(Color::Green),
        ),
        InputMode::Viewing => (build_viewing_help(), Style::default().fg(Color::Cyan)),
        InputMode::EditingDescription => (
            build_input_prompt("Editing description: ", &app.input_buffer),
            Style::default().fg(Color::Magenta),
        ),
        InputMode::AddingTag => (
            build_input_prompt("Adding tag: ", &app.input_buffer),
            Style::default().fg(Color::Blue),
        ),
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn build_normal_mode_help() -> Line<'static> {
    Line::from(vec![
        Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": new | "),
        Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": view | "),
        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": edit title | "),
        Span::styled("D", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": desc | "),
        Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": priority | "),
        Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": tag | "),
        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": delete | "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": quit"),
    ])
}

fn build_input_prompt<'a>(label: &'a str, buffer: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(buffer),
        Span::styled("█", Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to save | "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to cancel"),
    ])
}

fn build_viewing_help() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            "Viewing task details",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to close"),
    ])
}

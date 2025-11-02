//! Keyboard input handling for the Kanban TUI.

use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard events based on current input mode
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Creating => handle_creating_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
        InputMode::Viewing => handle_viewing_mode(app, key),
        InputMode::EditingDescription => handle_editing_description_mode(app, key),
        InputMode::AddingTag => handle_adding_tag_mode(app, key),
        InputMode::SelectingBoard => handle_selecting_board_mode(app, key),
        InputMode::CreatingBoard => handle_creating_board_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => return true, // Signal to quit
        KeyCode::Char('n') => app.start_creating(),
        KeyCode::Char('e') => app.start_editing(),
        KeyCode::Char('i') | KeyCode::Enter => app.start_viewing(),
        KeyCode::Char('p') => app.cycle_priority(),
        KeyCode::Char('D') => app.start_editing_description(),
        KeyCode::Char('t') => app.start_adding_tag(),
        KeyCode::Char('b') => app.start_board_selection(),
        KeyCode::Char('B') => app.start_creating_board(),
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
    }
    false
}

fn handle_creating_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => app.create_task(),
        KeyCode::Esc => app.cancel_creating(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                return true; // Quit on Ctrl+C
            }
            app.handle_char_input(c);
        }
        KeyCode::Backspace => app.handle_backspace(),
        _ => {}
    }
    false
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => app.save_edit(),
        KeyCode::Esc => app.cancel_editing(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                return true; // Quit on Ctrl+C
            }
            app.handle_char_input(c);
        }
        KeyCode::Backspace => app.handle_backspace(),
        _ => {}
    }
    false
}

fn handle_viewing_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('i') | KeyCode::Enter | KeyCode::Char('q') => {
            app.stop_viewing();
        }
        _ => {}
    }
    false
}

fn handle_editing_description_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => app.save_description(),
        KeyCode::Esc => app.cancel_editing_description(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                return true; // Quit on Ctrl+C
            }
            app.handle_char_input(c);
        }
        KeyCode::Backspace => app.handle_backspace(),
        _ => {}
    }
    false
}

fn handle_adding_tag_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => app.add_tag(),
        KeyCode::Esc => app.cancel_adding_tag(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                return true; // Quit on Ctrl+C
            }
            app.handle_char_input(c);
        }
        KeyCode::Backspace => app.handle_backspace(),
        _ => {}
    }
    false
}

fn handle_selecting_board_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => app.cancel_board_selection(),
        KeyCode::Enter => app.switch_to_selected_board(),
        KeyCode::Char('j') | KeyCode::Down => app.next_board_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_board_in_list(),
        KeyCode::Char('d') => app.delete_selected_board(),
        KeyCode::Char('n') | KeyCode::Char('B') => {
            app.cancel_board_selection();
            app.start_creating_board();
        }
        _ => {}
    }
    false
}

fn handle_creating_board_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => app.create_new_board(),
        KeyCode::Esc => app.cancel_creating_board(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                return true; // Quit on Ctrl+C
            }
            app.handle_char_input(c);
        }
        KeyCode::Backspace => app.handle_backspace(),
        _ => {}
    }
    false
}

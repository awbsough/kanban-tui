//! Application state management for the Kanban TUI.

use kanban_tui::{storage::Storage, Board};

/// Application input mode
#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Creating,
    Editing,
    Viewing,
    EditingDescription,
    AddingTag,
    SelectingBoard,
    CreatingBoard,
}

/// Application state
pub struct App {
    pub board: Board,
    pub selected_column: usize,
    pub selected_task_index: Option<usize>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub editing_task_id: Option<usize>,
    pub storage: Storage,
    pub current_board_name: String,
    pub available_boards: Vec<String>,
    pub selected_board_index: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let storage = Storage::new().expect("Failed to initialize storage");

        // Get active board name and load it
        let current_board_name = storage.get_active_board_name()
            .unwrap_or_else(|_| "default".to_string());

        let board = storage
            .load_board(&current_board_name)
            .ok()
            .flatten()
            .unwrap_or_else(|| Board::new(&current_board_name));

        // If board doesn't exist yet, save it
        if !storage.board_exists(&current_board_name) {
            let _ = storage.save_board(&current_board_name, &board);
        }

        // Load available boards
        let available_boards = storage.list_boards()
            .unwrap_or_else(|_| vec![current_board_name.clone()]);

        Self {
            board,
            selected_column: 0,
            selected_task_index: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_task_id: None,
            storage,
            current_board_name,
            available_boards,
            selected_board_index: None,
        }
    }

    /// Save the board to persistent storage
    pub fn save(&self) {
        if let Err(e) = self.storage.save_board(&self.current_board_name, &self.board) {
            eprintln!("Failed to save board: {}", e);
        }
    }

    // === Board Management ===

    pub fn start_board_selection(&mut self) {
        self.input_mode = InputMode::SelectingBoard;
        // Select current board in list
        self.selected_board_index = self.available_boards
            .iter()
            .position(|b| b == &self.current_board_name);
    }

    pub fn cancel_board_selection(&mut self) {
        self.input_mode = InputMode::Normal;
        self.selected_board_index = None;
    }

    pub fn next_board_in_list(&mut self) {
        if self.available_boards.is_empty() {
            return;
        }

        self.selected_board_index = Some(match self.selected_board_index {
            Some(idx) => (idx + 1) % self.available_boards.len(),
            None => 0,
        });
    }

    pub fn previous_board_in_list(&mut self) {
        if self.available_boards.is_empty() {
            return;
        }

        self.selected_board_index = Some(match self.selected_board_index {
            Some(idx) => {
                if idx > 0 {
                    idx - 1
                } else {
                    self.available_boards.len() - 1
                }
            }
            None => 0,
        });
    }

    pub fn switch_to_selected_board(&mut self) {
        if let Some(idx) = self.selected_board_index {
            if idx < self.available_boards.len() {
                let board_name = self.available_boards[idx].clone();
                self.input_buffer = board_name;
                self.switch_board();
            }
        }
        self.input_mode = InputMode::Normal;
        self.selected_board_index = None;
    }

    fn switch_board(&mut self) {
        let board_name = self.input_buffer.trim().to_string();

        if board_name.is_empty() {
            return;
        }

        // Save current board before switching
        self.save();

        // Load or create new board
        let new_board = self.storage
            .load_board(&board_name)
            .ok()
            .flatten()
            .unwrap_or_else(|| Board::new(&board_name));

        self.board = new_board;
        self.current_board_name = board_name.clone();

        // Save the new board and update metadata
        let _ = self.storage.save_board(&board_name, &self.board);
        let _ = self.storage.set_active_board_name(&board_name);

        // Refresh available boards list
        self.available_boards = self.storage.list_boards()
            .unwrap_or_else(|_| vec![board_name]);

        // Reset selections
        self.selected_column = 0;
        self.selected_task_index = None;
    }

    pub fn start_creating_board(&mut self) {
        self.input_mode = InputMode::CreatingBoard;
        self.input_buffer.clear();
    }

    pub fn create_new_board(&mut self) {
        if !self.input_buffer.is_empty() {
            // Create and switch to new board (board_name is in input_buffer)
            self.switch_board();
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn cancel_creating_board(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn delete_selected_board(&mut self) {
        if let Some(idx) = self.selected_board_index {
            if idx < self.available_boards.len() {
                let board_to_delete = self.available_boards[idx].clone();

                // Don't delete if it's the only board
                if self.available_boards.len() <= 1 {
                    return;
                }

                // Delete the board
                if let Ok(()) = self.storage.delete_board(&board_to_delete) {
                    // Refresh board list
                    self.available_boards = self.storage.list_boards()
                        .unwrap_or_else(|_| vec!["default".to_string()]);

                    // If we deleted the current board, switch to first available
                    if board_to_delete == self.current_board_name {
                        if let Some(first_board) = self.available_boards.first() {
                            let new_board = self.storage
                                .load_board(first_board)
                                .ok()
                                .flatten()
                                .unwrap_or_else(|| Board::new(first_board));

                            self.board = new_board;
                            self.current_board_name = first_board.clone();
                            let _ = self.storage.set_active_board_name(first_board);
                        }
                    }

                    // Adjust selection
                    if idx >= self.available_boards.len() {
                        self.selected_board_index = Some(self.available_boards.len().saturating_sub(1));
                    }
                }
            }
        }
    }

    // === Column Navigation ===

    pub fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % self.board.columns.len();
        self.update_task_selection();
    }

    pub fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        } else {
            self.selected_column = self.board.columns.len() - 1;
        }
        self.update_task_selection();
    }

    pub fn update_task_selection(&mut self) {
        // Auto-select first task if column has tasks, otherwise clear selection
        let task_count = self.board.columns[self.selected_column].tasks.len();
        self.selected_task_index = if task_count > 0 { Some(0) } else { None };
    }

    // === Task Navigation ===

    pub fn next_task(&mut self) {
        let task_count = self.board.columns[self.selected_column].tasks.len();
        if task_count == 0 {
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(idx) => (idx + 1) % task_count,
            None => 0,
        });
    }

    pub fn previous_task(&mut self) {
        let task_count = self.board.columns[self.selected_column].tasks.len();
        if task_count == 0 {
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(idx) => {
                if idx > 0 {
                    idx - 1
                } else {
                    task_count - 1
                }
            }
            None => 0,
        });
    }

    // === Task Management ===

    pub fn delete_selected_task(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            // Get task ID before deletion
            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;

                // Remove the task
                self.board.columns[self.selected_column].remove_task(task_id);

                // Adjust selection after deletion
                let new_task_count = self.board.columns[self.selected_column].tasks.len();
                if new_task_count == 0 {
                    self.selected_task_index = None;
                } else if task_idx >= new_task_count {
                    // If we deleted the last task, select the new last task
                    self.selected_task_index = Some(new_task_count - 1);
                }
                // Otherwise keep the same index (which now points to the next task)

                // Save after deletion
                self.save();
            }
        }
    }

    pub fn move_task_left(&mut self) {
        // Can't move left from first column
        if self.selected_column == 0 {
            return;
        }

        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;
                let from_column = self.selected_column;
                let to_column = self.selected_column - 1;

                // Move the task
                if self.board.move_task(from_column, to_column, task_id).is_ok() {
                    // Update selected column
                    self.selected_column = to_column;

                    // Find the moved task in the new column and select it
                    let new_task_index = self.board.columns[to_column]
                        .tasks
                        .iter()
                        .position(|t| t.id == task_id);
                    self.selected_task_index = new_task_index;

                    // Save after move
                    self.save();
                }
            }
        }
    }

    pub fn move_task_right(&mut self) {
        // Can't move right from last column
        if self.selected_column >= self.board.columns.len() - 1 {
            return;
        }

        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];

            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;
                let from_column = self.selected_column;
                let to_column = self.selected_column + 1;

                // Move the task
                if self.board.move_task(from_column, to_column, task_id).is_ok() {
                    // Update selected column
                    self.selected_column = to_column;

                    // Find the moved task in the new column and select it
                    let new_task_index = self.board.columns[to_column]
                        .tasks
                        .iter()
                        .position(|t| t.id == task_id);
                    self.selected_task_index = new_task_index;

                    // Save after move
                    self.save();
                }
            }
        }
    }

    // === Task Creation/Editing ===

    pub fn start_creating(&mut self) {
        self.input_mode = InputMode::Creating;
        self.input_buffer.clear();
    }

    pub fn create_task(&mut self) {
        if !self.input_buffer.is_empty() {
            let _ = self.board.add_task(self.selected_column, &self.input_buffer);
            self.input_buffer.clear();

            // Select the newly created task (last one in the column)
            let task_count = self.board.columns[self.selected_column].tasks.len();
            if task_count > 0 {
                self.selected_task_index = Some(task_count - 1);
            }

            // Save after creation
            self.save();
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_creating(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn start_editing(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];
            if task_idx < column.tasks.len() {
                let task = &column.tasks[task_idx];
                self.editing_task_id = Some(task.id);
                self.input_buffer = task.title.clone();
                self.input_mode = InputMode::Editing;
            }
        }
    }

    pub fn save_edit(&mut self) {
        if let Some(task_id) = self.editing_task_id {
            if !self.input_buffer.is_empty() {
                let _ = self.board.update_task_title(
                    self.selected_column,
                    task_id,
                    &self.input_buffer,
                );

                // Save after editing
                self.save();
            }
        }

        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    pub fn cancel_editing(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    pub fn handle_char_input(&mut self, c: char) {
        if self.input_mode == InputMode::Creating
            || self.input_mode == InputMode::Editing
            || self.input_mode == InputMode::EditingDescription
            || self.input_mode == InputMode::AddingTag
            || self.input_mode == InputMode::CreatingBoard
        {
            self.input_buffer.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.input_mode == InputMode::Creating
            || self.input_mode == InputMode::Editing
            || self.input_mode == InputMode::EditingDescription
            || self.input_mode == InputMode::AddingTag
            || self.input_mode == InputMode::CreatingBoard
        {
            self.input_buffer.pop();
        }
    }

    // === Task Viewing ===

    pub fn start_viewing(&mut self) {
        if self.selected_task_index.is_some() {
            self.input_mode = InputMode::Viewing;
        }
    }

    pub fn stop_viewing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    // === Task Metadata ===

    pub fn cycle_priority(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];
            if task_idx < column.tasks.len() {
                let task_id = column.tasks[task_idx].id;
                let _ = self.board.cycle_task_priority(self.selected_column, task_id);
                self.save();
            }
        }
    }

    pub fn start_editing_description(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            let column = &self.board.columns[self.selected_column];
            if task_idx < column.tasks.len() {
                let task = &column.tasks[task_idx];
                self.editing_task_id = Some(task.id);
                self.input_buffer = task.description.clone().unwrap_or_default();
                self.input_mode = InputMode::EditingDescription;
            }
        }
    }

    pub fn save_description(&mut self) {
        if let Some(task_id) = self.editing_task_id {
            let _ = self.board.update_task_description(
                self.selected_column,
                task_id,
                &self.input_buffer,
            );
            self.save();
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    pub fn cancel_editing_description(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_task_id = None;
    }

    pub fn start_adding_tag(&mut self) {
        if self.selected_task_index.is_some() {
            self.input_mode = InputMode::AddingTag;
            self.input_buffer.clear();
        }
    }

    pub fn add_tag(&mut self) {
        if let Some(task_idx) = self.selected_task_index {
            if !self.input_buffer.is_empty() {
                let column = &self.board.columns[self.selected_column];
                if task_idx < column.tasks.len() {
                    let task_id = column.tasks[task_idx].id;
                    let _ = self.board.add_task_tag(
                        self.selected_column,
                        task_id,
                        &self.input_buffer,
                    );
                    self.save();
                }
            }
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn cancel_adding_tag(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }
}

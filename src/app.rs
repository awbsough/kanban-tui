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
}

impl App {
    pub fn new() -> Self {
        let storage = Storage::new().expect("Failed to initialize storage");

        // Try to load existing board, or create new one
        let board = storage
            .load()
            .ok()
            .flatten()
            .unwrap_or_else(|| Board::new("My Kanban Board"));

        Self {
            board,
            selected_column: 0,
            selected_task_index: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_task_id: None,
            storage,
        }
    }

    /// Save the board to persistent storage
    pub fn save(&self) {
        if let Err(e) = self.storage.save(&self.board) {
            eprintln!("Failed to save board: {}", e);
        }
    }

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
        {
            self.input_buffer.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.input_mode == InputMode::Creating
            || self.input_mode == InputMode::Editing
            || self.input_mode == InputMode::EditingDescription
            || self.input_mode == InputMode::AddingTag
        {
            self.input_buffer.pop();
        }
    }

    pub fn start_viewing(&mut self) {
        if self.selected_task_index.is_some() {
            self.input_mode = InputMode::Viewing;
        }
    }

    pub fn stop_viewing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

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

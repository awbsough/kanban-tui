//! Board type for managing Kanban columns and tasks.

use crate::{Column, Task};
use serde::{Deserialize, Serialize};

/// Represents a Kanban board with multiple columns.
///
/// A board contains a collection of columns (default: "To Do", "In Progress", "Done")
/// and manages task IDs automatically. Tasks are organized within columns and can be
/// moved between them.
///
/// # Examples
///
/// ```
/// use kanban_tui::Board;
///
/// // Create a board with default columns
/// let mut board = Board::new("My Project".to_string());
/// assert_eq!(board.columns.len(), 3);
///
/// // Add a task
/// let task_id = board.add_task(0, "Write tests".to_string()).unwrap();
///
/// // Move task to next column
/// board.move_task(0, 1, task_id).unwrap();
/// assert_eq!(board.columns[1].tasks.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub name: String,
    pub columns: Vec<Column>,
    next_task_id: usize,
}

impl Board {
    /// Creates a new board with default columns (To Do, In Progress, Done).
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Board;
    ///
    /// let board = Board::new("Sprint 1".to_string());
    /// assert_eq!(board.name, "Sprint 1");
    /// assert_eq!(board.columns.len(), 3);
    /// assert_eq!(board.columns[0].name, "To Do");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: vec![
                Column::new("To Do"),
                Column::new("In Progress"),
                Column::new("Done"),
            ],
            next_task_id: 1,
        }
    }

    /// Creates a new board with custom columns
    pub fn with_columns(name: impl Into<String>, column_names: Vec<String>) -> Self {
        let columns = column_names.into_iter().map(Column::new).collect();
        Self {
            name: name.into(),
            columns,
            next_task_id: 1,
        }
    }

    /// Adds a new task to the specified column.
    ///
    /// Returns the ID of the newly created task.
    ///
    /// # Errors
    ///
    /// Returns an error if the column index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Board;
    ///
    /// let mut board = Board::new("Project".to_string());
    ///
    /// // Add task to first column
    /// let task_id = board.add_task(0, "Implement login".to_string()).unwrap();
    /// assert_eq!(board.columns[0].tasks.len(), 1);
    ///
    /// // Invalid column index returns error
    /// let result = board.add_task(99, "Task".to_string());
    /// assert!(result.is_err());
    /// ```
    pub fn add_task(&mut self, column_index: usize, title: impl Into<String>) -> Result<usize, String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = Task::new(task_id, title);
        self.columns[column_index].add_task(task);

        Ok(task_id)
    }

    /// Moves a task from one column to another.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Either column index is out of bounds
    /// - The task is not found in the source column
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Board;
    ///
    /// let mut board = Board::new("Project".to_string());
    /// let task_id = board.add_task(0, "Task".to_string()).unwrap();
    ///
    /// // Move from "To Do" (0) to "In Progress" (1)
    /// board.move_task(0, 1, task_id).unwrap();
    /// assert_eq!(board.columns[0].tasks.len(), 0);
    /// assert_eq!(board.columns[1].tasks.len(), 1);
    /// ```
    pub fn move_task(
        &mut self,
        from_column: usize,
        to_column: usize,
        task_id: usize,
    ) -> Result<(), String> {
        if from_column >= self.columns.len() || to_column >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[from_column]
            .remove_task(task_id)
            .ok_or("Task not found in source column")?;

        self.columns[to_column].add_task(task);
        Ok(())
    }

    /// Updates the title of a task in a specified column
    pub fn update_task_title(
        &mut self,
        column_index: usize,
        task_id: usize,
        new_title: impl Into<String>,
    ) -> Result<(), String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[column_index]
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or("Task not found in column")?;

        task.update_title(new_title);
        Ok(())
    }

    /// Updates the description of a task in a specified column
    pub fn update_task_description(
        &mut self,
        column_index: usize,
        task_id: usize,
        description: impl Into<String>,
    ) -> Result<(), String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[column_index]
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or("Task not found in column")?;

        task.set_description(description);
        Ok(())
    }

    /// Cycles the priority of a task in a specified column
    pub fn cycle_task_priority(
        &mut self,
        column_index: usize,
        task_id: usize,
    ) -> Result<(), String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[column_index]
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or("Task not found in column")?;

        task.cycle_priority();
        Ok(())
    }

    /// Adds a tag to a task in a specified column
    pub fn add_task_tag(
        &mut self,
        column_index: usize,
        task_id: usize,
        tag: impl Into<String>,
    ) -> Result<(), String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[column_index]
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or("Task not found in column")?;

        task.add_tag(tag);
        Ok(())
    }

    /// Sets the due date of a task in a specified column
    pub fn set_task_due_date(
        &mut self,
        column_index: usize,
        task_id: usize,
        due_date: Option<String>,
    ) -> Result<(), String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task = self.columns[column_index]
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or("Task not found in column")?;

        task.set_due_date(due_date);
        Ok(())
    }

    /// Gets a reference to a task by ID, searching all columns
    pub fn get_task(&self, task_id: usize) -> Option<(&Task, usize)> {
        for (col_idx, column) in self.columns.iter().enumerate() {
            if let Some(task) = column.tasks.iter().find(|t| t.id == task_id) {
                return Some((task, col_idx));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new("My Board");
        assert_eq!(board.name, "My Board");
        assert_eq!(board.columns.len(), 3);
        assert_eq!(board.columns[0].name, "To Do");
        assert_eq!(board.columns[1].name, "In Progress");
        assert_eq!(board.columns[2].name, "Done");
    }

    #[test]
    fn test_board_add_task() {
        let mut board = Board::new("Test");
        let result = board.add_task(0, "New task");

        assert!(result.is_ok());
        assert_eq!(board.columns[0].tasks.len(), 1);
        assert_eq!(board.columns[0].tasks[0].title, "New task");
    }

    #[test]
    fn test_board_move_task() {
        let mut board = Board::new("Test");
        let task_id = board.add_task(0, "Task to move").unwrap();

        let result = board.move_task(0, 1, task_id);
        assert!(result.is_ok());
        assert_eq!(board.columns[0].tasks.len(), 0);
        assert_eq!(board.columns[1].tasks.len(), 1);
        assert_eq!(board.columns[1].tasks[0].title, "Task to move");
    }

    #[test]
    fn test_board_move_task_invalid_column() {
        let mut board = Board::new("Test");
        let task_id = board.add_task(0, "Task").unwrap();

        let result = board.move_task(0, 10, task_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_board_update_task_title() {
        let mut board = Board::new("Test");
        let task_id = board.add_task(0, "Original Title").unwrap();

        // Update the task title
        let result = board.update_task_title(0, task_id, "Updated Title");
        assert!(result.is_ok());

        // Verify the title was updated
        assert_eq!(board.columns[0].tasks[0].title, "Updated Title");
    }

    #[test]
    fn test_board_update_task_title_invalid_column() {
        let mut board = Board::new("Test");
        let task_id = board.add_task(0, "Task").unwrap();

        // Try to update task in non-existent column
        let result = board.update_task_title(10, task_id, "New Title");
        assert!(result.is_err());
    }

    #[test]
    fn test_board_update_task_title_invalid_task() {
        let mut board = Board::new("Test");
        board.add_task(0, "Task").unwrap();

        // Try to update non-existent task
        let result = board.update_task_title(0, 9999, "New Title");
        assert!(result.is_err());
    }
}

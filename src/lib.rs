/// Core library for the Kanban TUI application.
///
/// This library provides the data structures and business logic
/// for managing Kanban boards, separate from the UI layer.

use serde::{Deserialize, Serialize};

pub mod storage;

/// Represents a single task in the Kanban board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub description: Option<String>,
}

impl Task {
    /// Creates a new task with the given title
    pub fn new(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
        }
    }

    /// Creates a new task with a title and description
    pub fn with_description(id: usize, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description: Some(description),
        }
    }
}

/// Represents a column in the Kanban board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Column {
    /// Creates a new empty column with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            tasks: Vec::new(),
        }
    }

    /// Adds a task to the column
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Removes a task by ID and returns it if found
    pub fn remove_task(&mut self, task_id: usize) -> Option<Task> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }
}

/// Represents a Kanban board with multiple columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub name: String,
    pub columns: Vec<Column>,
    next_task_id: usize,
}

impl Board {
    /// Creates a new board with default columns (To Do, In Progress, Done)
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: vec![
                Column::new("To Do".to_string()),
                Column::new("In Progress".to_string()),
                Column::new("Done".to_string()),
            ],
            next_task_id: 1,
        }
    }

    /// Creates a new board with custom columns
    pub fn with_columns(name: String, column_names: Vec<String>) -> Self {
        let columns = column_names.into_iter().map(Column::new).collect();
        Self {
            name,
            columns,
            next_task_id: 1,
        }
    }

    /// Adds a new task to the specified column
    pub fn add_task(&mut self, column_index: usize, title: String) -> Result<usize, String> {
        if column_index >= self.columns.len() {
            return Err("Column index out of bounds".to_string());
        }

        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = Task::new(task_id, title);
        self.columns[column_index].add_task(task);

        Ok(task_id)
    }

    /// Moves a task from one column to another
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let task = Task::new(1, "Test task".to_string());
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, None);
    }

    #[test]
    fn test_create_task_with_description() {
        let task = Task::with_description(
            1,
            "Test task".to_string(),
            "Description".to_string(),
        );
        assert_eq!(task.description, Some("Description".to_string()));
    }

    #[test]
    fn test_column_add_remove_task() {
        let mut column = Column::new("To Do".to_string());
        let task = Task::new(1, "Test".to_string());

        column.add_task(task.clone());
        assert_eq!(column.tasks.len(), 1);

        let removed = column.remove_task(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), task);
        assert_eq!(column.tasks.len(), 0);
    }

    #[test]
    fn test_board_creation() {
        let board = Board::new("My Board".to_string());
        assert_eq!(board.name, "My Board");
        assert_eq!(board.columns.len(), 3);
        assert_eq!(board.columns[0].name, "To Do");
        assert_eq!(board.columns[1].name, "In Progress");
        assert_eq!(board.columns[2].name, "Done");
    }

    #[test]
    fn test_board_add_task() {
        let mut board = Board::new("Test".to_string());
        let result = board.add_task(0, "New task".to_string());

        assert!(result.is_ok());
        assert_eq!(board.columns[0].tasks.len(), 1);
        assert_eq!(board.columns[0].tasks[0].title, "New task");
    }

    #[test]
    fn test_board_move_task() {
        let mut board = Board::new("Test".to_string());
        let task_id = board.add_task(0, "Task to move".to_string()).unwrap();

        let result = board.move_task(0, 1, task_id);
        assert!(result.is_ok());
        assert_eq!(board.columns[0].tasks.len(), 0);
        assert_eq!(board.columns[1].tasks.len(), 1);
        assert_eq!(board.columns[1].tasks[0].title, "Task to move");
    }

    #[test]
    fn test_board_move_task_invalid_column() {
        let mut board = Board::new("Test".to_string());
        let task_id = board.add_task(0, "Task".to_string()).unwrap();

        let result = board.move_task(0, 10, task_id);
        assert!(result.is_err());
    }
}

//! Column type for organizing tasks in Kanban boards.

use crate::Task;
use serde::{Deserialize, Serialize};

/// Represents a column in the Kanban board.
///
/// A column contains a name and a list of tasks. Common column names include
/// "To Do", "In Progress", and "Done", but columns can have any name.
///
/// # Examples
///
/// ```
/// use kanban_tui::{Column, Task};
///
/// let mut column = Column::new("To Do".to_string());
/// assert_eq!(column.name, "To Do");
/// assert!(column.tasks.is_empty());
///
/// // Add a task
/// let task = Task::new(1, "My task".to_string());
/// column.add_task(task);
/// assert_eq!(column.tasks.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Column {
    /// Creates a new empty column with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_add_remove_task() {
        let mut column = Column::new("To Do");
        let task = Task::new(1, "Test");

        column.add_task(task.clone());
        assert_eq!(column.tasks.len(), 1);

        let removed = column.remove_task(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), task);
        assert_eq!(column.tasks.len(), 0);
    }
}

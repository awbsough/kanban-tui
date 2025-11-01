//! # Kanban TUI Library
//!
//! A library for managing Kanban boards with support for tasks, columns, priorities,
//! tags, and persistent storage.
//!
//! This library provides the core data structures and business logic for building
//! Kanban-style task management applications. It is designed to be UI-agnostic,
//! allowing you to build terminal UIs, web interfaces, or other frontends on top of it.
//!
//! ## Features
//!
//! - **Task Management**: Create, edit, and organize tasks with titles, descriptions, and metadata
//! - **Priority System**: Assign priorities (High, Medium, Low, None) to tasks
//! - **Tags**: Organize tasks with custom tags
//! - **Columns**: Organize tasks into customizable columns (default: To Do, In Progress, Done)
//! - **Persistent Storage**: Save and load boards from JSON files with platform-specific locations
//! - **Type Safety**: Comprehensive error handling with proper Result types
//!
//! ## Quick Start
//!
//! ```rust
//! use kanban_tui::{Board, Task, Priority};
//!
//! // Create a new board with default columns
//! let mut board = Board::new("My Project".to_string());
//!
//! // Add a task to the first column (To Do)
//! let task_id = board.add_task(0, "Implement feature".to_string()).unwrap();
//!
//! // Move the task to the second column (In Progress)
//! board.move_task(0, 1, task_id).unwrap();
//!
//! // Update task details
//! board.update_task_title(1, task_id, "Implement awesome feature".to_string()).unwrap();
//! board.cycle_task_priority(1, task_id).unwrap();
//! ```
//!
//! ## Persistent Storage Example
//!
//! ```rust,no_run
//! use kanban_tui::{Board, storage::Storage};
//!
//! // Create or load a board
//! let storage = Storage::new().expect("Failed to initialize storage");
//! let mut board = storage.load()
//!     .ok()
//!     .flatten()
//!     .unwrap_or_else(|| Board::new("My Board".to_string()));
//!
//! // Make changes
//! board.add_task(0, "New task".to_string()).unwrap();
//!
//! // Save the board
//! storage.save(&board).expect("Failed to save board");
//! ```
//!
//! ## Architecture
//!
//! The library is organized around three main types:
//!
//! - [`Board`]: The top-level container that holds columns and manages tasks
//! - [`Column`]: A vertical section of the board containing tasks
//! - [`Task`]: An individual work item with metadata
//!
//! The [`storage`] module provides persistence functionality using JSON files
//! stored in platform-specific configuration directories.

use serde::{Deserialize, Serialize};

pub mod storage;

/// Priority level for tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    // Ordered from highest to lowest priority (High > Medium > Low > None)
    High,
    Medium,
    Low,
    None,
}

impl Priority {
    /// Get the next priority level (cycles through all levels)
    pub fn next(&self) -> Self {
        match self {
            Priority::None => Priority::Low,
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::None,
        }
    }

    /// Get a display symbol for the priority
    pub fn symbol(&self) -> &str {
        match self {
            Priority::High => "!!",
            Priority::Medium => "!",
            Priority::Low => "·",
            Priority::None => "",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::None
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "High"),
            Priority::Medium => write!(f, "Medium"),
            Priority::Low => write!(f, "Low"),
            Priority::None => write!(f, "None"),
        }
    }
}

/// Represents a single task in the Kanban board.
///
/// A task contains a unique ID, title, optional description, priority level,
/// tags, and timestamps for creation and updates.
///
/// # Examples
///
/// ```
/// use kanban_tui::{Task, Priority};
///
/// // Create a simple task
/// let task = Task::new(1, "Write documentation".to_string());
/// assert_eq!(task.id, 1);
/// assert_eq!(task.title, "Write documentation");
/// assert_eq!(task.priority, Priority::None);
///
/// // Create a task with description
/// let task = Task::with_description(
///     2,
///     "Review PR".to_string(),
///     "Check code quality and tests".to_string()
/// );
/// assert!(task.description.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "current_timestamp")]
    pub created_at: String,
    #[serde(default = "current_timestamp")]
    pub updated_at: String,
    #[serde(default)]
    pub due_date: Option<String>,
}

/// Helper function for serde default
fn current_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Task {
    /// Creates a new task with the given title.
    ///
    /// The task is initialized with default values: no description, no priority,
    /// empty tags list, and timestamps set to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Task;
    ///
    /// let task = Task::new(1, "Implement feature".to_string());
    /// assert_eq!(task.id, 1);
    /// assert_eq!(task.title, "Implement feature");
    /// assert!(task.description.is_none());
    /// assert!(task.tags.is_empty());
    /// ```
    pub fn new(id: usize, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            priority: Priority::None,
            tags: Vec::new(),
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            due_date: None,
        }
    }

    /// Creates a new task with a title and description.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Task;
    ///
    /// let task = Task::with_description(
    ///     1,
    ///     "Fix bug".to_string(),
    ///     "The submit button doesn't work on mobile".to_string()
    /// );
    /// assert_eq!(task.description, Some("The submit button doesn't work on mobile".to_string()));
    /// ```
    pub fn with_description(id: usize, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: Some(description.into()),
            priority: Priority::None,
            tags: Vec::new(),
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            due_date: None,
        }
    }

    /// Updates the description of the task
    pub fn set_description(&mut self, description: impl Into<String>) {
        let desc = description.into();
        self.description = if desc.is_empty() {
            None
        } else {
            Some(desc)
        };
        self.updated_at = current_timestamp();
    }

    /// Sets the priority of the task
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = current_timestamp();
    }

    /// Cycles to the next priority level.
    ///
    /// Priority cycles through: None → Low → Medium → High → None
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::{Task, Priority};
    ///
    /// let mut task = Task::new(1, "Task".to_string());
    /// assert_eq!(task.priority, Priority::None);
    ///
    /// task.cycle_priority();
    /// assert_eq!(task.priority, Priority::Low);
    ///
    /// task.cycle_priority();
    /// assert_eq!(task.priority, Priority::Medium);
    /// ```
    pub fn cycle_priority(&mut self) {
        self.priority = self.priority.next();
        self.updated_at = current_timestamp();
    }

    /// Adds a tag to the task if it doesn't already exist.
    ///
    /// Empty tags are ignored. Duplicate tags are not added.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_tui::Task;
    ///
    /// let mut task = Task::new(1, "Task".to_string());
    /// task.add_tag("urgent".to_string());
    /// task.add_tag("backend".to_string());
    /// task.add_tag("urgent".to_string()); // Duplicate, won't be added
    ///
    /// assert_eq!(task.tags.len(), 2);
    /// assert!(task.tags.contains(&"urgent".to_string()));
    /// ```
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag_str = tag.into();
        if !self.tags.contains(&tag_str) && !tag_str.is_empty() {
            self.tags.push(tag_str);
            self.updated_at = current_timestamp();
        }
    }

    /// Removes a tag from the task
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = current_timestamp();
        }
    }

    /// Sets the due date for the task
    pub fn set_due_date(&mut self, due_date: Option<String>) {
        self.due_date = due_date;
        self.updated_at = current_timestamp();
    }

    /// Updates the title and timestamp
    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = current_timestamp();
    }
}

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
    fn test_create_task() {
        let task = Task::new(1, "Test task");
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, None);
    }

    #[test]
    fn test_create_task_with_description() {
        let task = Task::with_description(
            1,
            "Test task",
            "Description",
        );
        assert_eq!(task.description, Some("Description".to_string()));
    }

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

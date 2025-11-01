//! Task and Priority types for Kanban boards.

use serde::{Deserialize, Serialize};

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
        let task = Task::with_description(1, "Test task", "Description");
        assert_eq!(task.description, Some("Description".to_string()));
    }
}

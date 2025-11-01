//! Persistent storage for Kanban boards.
//!
//! This module provides functionality to save and load boards from JSON files
//! stored in platform-specific configuration directories.
//!
//! # Examples
//!
//! ```rust,no_run
//! use kanban_tui::{Board, storage::Storage};
//!
//! // Create storage with default location
//! let storage = Storage::new().expect("Failed to initialize storage");
//!
//! // Create a board
//! let mut board = Board::new("My Project".to_string());
//! board.add_task(0, "Task 1".to_string()).unwrap();
//!
//! // Save the board
//! storage.save(&board).expect("Failed to save");
//!
//! // Load the board
//! let loaded = storage.load().expect("Failed to load");
//! assert!(loaded.is_some());
//! ```

use crate::Board;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Errors that can occur during storage operations.
#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Serialization(serde_json::Error),
    ConfigDirNotFound,
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err)
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(err) => write!(f, "IO error: {}", err),
            StorageError::Serialization(err) => write!(f, "Serialization error: {}", err),
            StorageError::ConfigDirNotFound => write!(f, "Could not find config directory"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Handles persistent storage of Kanban boards.
///
/// Storage manages reading and writing boards to JSON files in platform-specific
/// configuration directories:
/// - Linux: `~/.config/kanban-tui/board.json`
/// - macOS: `~/Library/Application Support/kanban-tui/board.json`
/// - Windows: `%APPDATA%\kanban-tui\board.json`
///
/// # Examples
///
/// ```rust,no_run
/// use kanban_tui::{Board, storage::Storage};
///
/// let storage = Storage::new().expect("Failed to create storage");
/// let mut board = Board::new("Project".to_string());
///
/// // Save and load
/// storage.save(&board).expect("Failed to save");
/// let loaded = storage.load().expect("Failed to load").unwrap();
/// assert_eq!(loaded.name, "Project");
/// ```
pub struct Storage {
    file_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance with the default file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform-specific config directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kanban_tui::storage::Storage;
    ///
    /// let storage = Storage::new().expect("Failed to create storage");
    /// println!("Storage path: {:?}", storage.file_path());
    /// ```
    pub fn new() -> Result<Self, StorageError> {
        let file_path = Self::default_file_path()?;
        Ok(Storage { file_path })
    }

    /// Create a new Storage instance with a custom file path (useful for testing)
    pub fn with_path(file_path: PathBuf) -> Self {
        Storage { file_path }
    }

    /// Get the default file path for storing the board
    pub fn default_file_path() -> Result<PathBuf, StorageError> {
        let config_dir = dirs::config_dir().ok_or(StorageError::ConfigDirNotFound)?;
        let app_dir = config_dir.join("kanban-tui");
        Ok(app_dir.join("board.json"))
    }

    /// Ensure the storage directory exists
    fn ensure_dir_exists(&self) -> Result<(), StorageError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Save a board to the storage file.
    ///
    /// Creates the storage directory if it doesn't exist. The board is serialized
    /// to pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory cannot be created
    /// - The board cannot be serialized
    /// - The file cannot be written
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kanban_tui::{Board, storage::Storage};
    ///
    /// let storage = Storage::new().unwrap();
    /// let board = Board::new("Project".to_string());
    /// storage.save(&board).expect("Failed to save board");
    /// ```
    pub fn save(&self, board: &Board) -> Result<(), StorageError> {
        self.ensure_dir_exists()?;
        let json = serde_json::to_string_pretty(board)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }

    /// Load a board from the storage file.
    ///
    /// Returns `None` if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file exists but cannot be read
    /// - The JSON content is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kanban_tui::{Board, storage::Storage};
    ///
    /// let storage = Storage::new().unwrap();
    ///
    /// match storage.load() {
    ///     Ok(Some(board)) => println!("Loaded board: {}", board.name),
    ///     Ok(None) => println!("No saved board found"),
    ///     Err(e) => eprintln!("Error loading board: {}", e),
    /// }
    /// ```
    pub fn load(&self) -> Result<Option<Board>, StorageError> {
        if !self.file_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&self.file_path)?;
        let board = serde_json::from_str(&json)?;
        Ok(Some(board))
    }

    /// Check if the storage file exists
    pub fn exists(&self) -> bool {
        self.file_path.exists()
    }

    /// Get the file path being used
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Compile-time test to ensure StorageError is Send + Sync
    // This is required for proper error handling in async/multi-threaded contexts
    #[allow(dead_code)]
    fn assert_error_traits() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<StorageError>();
        assert_sync::<StorageError>();
    }

    fn temp_storage() -> Storage {
        let temp_dir = env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_file = temp_dir.join(format!("kanban-test-{}.json", timestamp));
        Storage::with_path(test_file)
    }

    #[test]
    fn test_save_and_load() {
        let storage = temp_storage();
        let mut board = Board::new("Test Board".to_string());
        board.add_task(0, "Task 1".to_string()).unwrap();
        board.add_task(1, "Task 2".to_string()).unwrap();

        // Save the board
        storage.save(&board).unwrap();

        // Load it back
        let loaded = storage.load().unwrap();
        assert!(loaded.is_some());
        let loaded_board = loaded.unwrap();

        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns[0].tasks.len(), 1);
        assert_eq!(loaded_board.columns[0].tasks[0].title, "Task 1");
        assert_eq!(loaded_board.columns[1].tasks.len(), 1);
        assert_eq!(loaded_board.columns[1].tasks[0].title, "Task 2");

        // Cleanup
        fs::remove_file(storage.file_path()).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let storage = temp_storage();

        // Should return None for non-existent file
        let result = storage.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_exists() {
        let storage = temp_storage();
        let board = Board::new("Test".to_string());

        assert!(!storage.exists());

        storage.save(&board).unwrap();
        assert!(storage.exists());

        // Cleanup
        fs::remove_file(storage.file_path()).ok();
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let nested_path = temp_dir.join(format!("kanban-test-dir-{}", timestamp));
        let file_path = nested_path.join("board.json");

        let storage = Storage::with_path(file_path.clone());
        let board = Board::new("Test".to_string());

        storage.save(&board).unwrap();
        assert!(file_path.exists());

        // Cleanup
        fs::remove_dir_all(&nested_path).ok();
    }
}

//! Persistent storage for Kanban boards.
//!
//! This module provides functionality to save and load multiple boards from JSON files
//! stored in platform-specific configuration directories.

use crate::Board;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Errors that can occur during storage operations.
#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Serialization(serde_json::Error),
    ConfigDirNotFound,
    BoardNotFound(String),
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
            StorageError::BoardNotFound(name) => write!(f, "Board not found: {}", name),
        }
    }
}

impl std::error::Error for StorageError {}

/// Metadata for tracking active board and board list
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    active_board: String,
    #[serde(default)]
    boards: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            active_board: "default".to_string(),
            boards: vec!["default".to_string()],
        }
    }
}

/// Handles persistent storage of multiple Kanban boards.
///
/// Storage manages reading and writing boards to JSON files in platform-specific
/// configuration directories:
/// - Linux: `~/.config/kanban-tui/boards/`
/// - macOS: `~/Library/Application Support/kanban-tui/boards/`
/// - Windows: `%APPDATA%\kanban-tui\boards\`
pub struct Storage {
    boards_dir: PathBuf,
    metadata_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance with the default directory path.
    pub fn new() -> Result<Self, StorageError> {
        let config_dir = dirs::config_dir().ok_or(StorageError::ConfigDirNotFound)?;
        let app_dir = config_dir.join("kanban-tui");
        let boards_dir = app_dir.join("boards");
        let metadata_path = app_dir.join("metadata.json");

        let storage = Storage {
            boards_dir,
            metadata_path,
        };

        // Ensure directory exists and migrate old format if needed
        storage.ensure_dirs_exist()?;
        storage.migrate_old_format()?;

        Ok(storage)
    }

    /// Create a Storage instance with custom paths (useful for testing)
    pub fn with_path(base_dir: PathBuf) -> Self {
        let boards_dir = base_dir.join("boards");
        let metadata_path = base_dir.join("metadata.json");

        Storage {
            boards_dir,
            metadata_path,
        }
    }

    /// Ensure the storage directories exist
    fn ensure_dirs_exist(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.boards_dir)?;
        Ok(())
    }

    /// Migrate old single-board format to new multi-board format
    fn migrate_old_format(&self) -> Result<(), StorageError> {
        let old_board_path = self.boards_dir.parent().unwrap().join("board.json");

        // If old format exists and new format doesn't, migrate
        if old_board_path.exists() && !self.metadata_path.exists() {
            // Move old board.json to boards/default.json
            let default_board_path = self.board_path("default");
            fs::rename(&old_board_path, &default_board_path)?;

            // Create metadata
            let metadata = Metadata::default();
            self.save_metadata(&metadata)?;
        }

        // Ensure metadata exists even if no migration happened
        if !self.metadata_path.exists() {
            let metadata = Metadata::default();
            self.save_metadata(&metadata)?;
        }

        Ok(())
    }

    /// Get the file path for a specific board
    fn board_path(&self, name: &str) -> PathBuf {
        let safe_name = Self::sanitize_board_name(name);
        self.boards_dir.join(format!("{}.json", safe_name))
    }

    /// Sanitize board name for filesystem safety
    fn sanitize_board_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect()
    }

    /// Load metadata
    fn load_metadata(&self) -> Result<Metadata, StorageError> {
        if !self.metadata_path.exists() {
            return Ok(Metadata::default());
        }

        let json = fs::read_to_string(&self.metadata_path)?;
        let metadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Save metadata
    fn save_metadata(&self, metadata: &Metadata) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(&self.metadata_path, json)?;
        Ok(())
    }

    /// Get the name of the currently active board
    pub fn get_active_board_name(&self) -> Result<String, StorageError> {
        let metadata = self.load_metadata()?;
        Ok(metadata.active_board)
    }

    /// Set the active board name
    pub fn set_active_board_name(&self, name: &str) -> Result<(), StorageError> {
        let mut metadata = self.load_metadata()?;
        metadata.active_board = name.to_string();

        // Ensure board exists in the list
        if !metadata.boards.contains(&name.to_string()) {
            metadata.boards.push(name.to_string());
        }

        self.save_metadata(&metadata)?;
        Ok(())
    }

    /// List all available boards
    pub fn list_boards(&self) -> Result<Vec<String>, StorageError> {
        let metadata = self.load_metadata()?;
        Ok(metadata.boards)
    }

    /// Load a specific board by name
    pub fn load_board(&self, name: &str) -> Result<Option<Board>, StorageError> {
        let board_path = self.board_path(name);

        if !board_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&board_path)?;
        let board = serde_json::from_str(&json)?;
        Ok(Some(board))
    }

    /// Save a specific board
    pub fn save_board(&self, name: &str, board: &Board) -> Result<(), StorageError> {
        self.ensure_dirs_exist()?;

        let board_path = self.board_path(name);
        let json = serde_json::to_string_pretty(board)?;
        fs::write(&board_path, json)?;

        // Ensure board is in metadata
        let mut metadata = self.load_metadata()?;
        if !metadata.boards.contains(&name.to_string()) {
            metadata.boards.push(name.to_string());
            self.save_metadata(&metadata)?;
        }

        Ok(())
    }

    /// Delete a board
    pub fn delete_board(&self, name: &str) -> Result<(), StorageError> {
        let board_path = self.board_path(name);

        if board_path.exists() {
            fs::remove_file(&board_path)?;
        }

        // Remove from metadata
        let mut metadata = self.load_metadata()?;
        metadata.boards.retain(|b| b != name);

        // If we deleted the active board, switch to default or first available
        if metadata.active_board == name {
            metadata.active_board = metadata.boards.first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
        }

        self.save_metadata(&metadata)?;
        Ok(())
    }

    /// Check if a board exists
    pub fn board_exists(&self, name: &str) -> bool {
        self.board_path(name).exists()
    }

    /// Legacy method for backward compatibility - loads active board
    #[deprecated(note = "Use load_board with get_active_board_name instead")]
    pub fn load(&self) -> Result<Option<Board>, StorageError> {
        let active_name = self.get_active_board_name()?;
        self.load_board(&active_name)
    }

    /// Legacy method for backward compatibility - saves to active board
    #[deprecated(note = "Use save_board with board name instead")]
    pub fn save(&self, board: &Board) -> Result<(), StorageError> {
        let active_name = self.get_active_board_name()?;
        self.save_board(&active_name, board)
    }

    /// Get the file path being used (legacy)
    #[deprecated(note = "Storage now uses multiple files")]
    pub fn file_path(&self) -> &PathBuf {
        &self.metadata_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_storage() -> Storage {
        let temp_dir = env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = temp_dir.join(format!("kanban-test-{}", timestamp));
        Storage::with_path(test_dir)
    }

    #[test]
    fn test_save_and_load_board() {
        let storage = temp_storage();
        storage.ensure_dirs_exist().unwrap();

        let mut board = Board::new("Test Board");
        board.add_task(0, "Task 1").unwrap();

        storage.save_board("test", &board).unwrap();

        let loaded = storage.load_board("test").unwrap();
        assert!(loaded.is_some());
        let loaded_board = loaded.unwrap();
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns[0].tasks.len(), 1);
    }

    #[test]
    fn test_list_boards() {
        let storage = temp_storage();
        storage.ensure_dirs_exist().unwrap();

        let board1 = Board::new("Board 1");
        let board2 = Board::new("Board 2");

        storage.save_board("board1", &board1).unwrap();
        storage.save_board("board2", &board2).unwrap();

        let boards = storage.list_boards().unwrap();
        assert!(boards.contains(&"board1".to_string()));
        assert!(boards.contains(&"board2".to_string()));
    }

    #[test]
    fn test_active_board_tracking() {
        let storage = temp_storage();
        storage.ensure_dirs_exist().unwrap();

        storage.set_active_board_name("my-board").unwrap();
        let active = storage.get_active_board_name().unwrap();
        assert_eq!(active, "my-board");
    }

    #[test]
    fn test_delete_board() {
        let storage = temp_storage();
        storage.ensure_dirs_exist().unwrap();

        let board = Board::new("To Delete");
        storage.save_board("deleteme", &board).unwrap();
        assert!(storage.board_exists("deleteme"));

        storage.delete_board("deleteme").unwrap();
        assert!(!storage.board_exists("deleteme"));
    }

    #[test]
    fn test_sanitize_board_name() {
        assert_eq!(Storage::sanitize_board_name("My Board!"), "My-Board-");
        assert_eq!(Storage::sanitize_board_name("test@123"), "test-123");
        assert_eq!(Storage::sanitize_board_name("valid_name-123"), "valid_name-123");
    }
}

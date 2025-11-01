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

mod task;
mod column;
mod board;

pub mod storage;

// Re-export main types
pub use task::{Task, Priority};
pub use column::Column;
pub use board::Board;

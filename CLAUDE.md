# Kanban TUI

## Project Overview

A Terminal User Interface (TUI) application for managing Kanban boards, written in Rust. This tool allows users to organize tasks using the Kanban methodology directly from the terminal.

## Architecture

### Core Components

- **TUI Layer** (`ratatui` + `crossterm`): Handles rendering and user input
- **State Management**: Board state, columns, and tasks
- **Storage**: Persistence layer for saving/loading boards (future: JSON/SQLite)
- **Core Logic**: Business logic for task management, movement, and validation

### Project Structure

```
kanban-tui/
├── src/
│   ├── main.rs          # Application entry point and TUI implementation
│   ├── lib.rs           # Public library interface
│   └── storage.rs       # Persistence layer (JSON storage)
├── tests/               # Integration tests
└── examples/            # Example usage (future)
```

## Development Environment

### Nix + direnv Setup

This project uses Nix flakes for reproducible development environments:

1. **Prerequisites**: Install Nix with flakes enabled and direnv
2. **Automatic activation**: Run `direnv allow` in the project root
3. The environment provides:
   - Rust stable toolchain (rustc, cargo)
   - rust-analyzer for IDE support
   - cargo-watch for development
   - Additional development tools

### Development Workflow

```bash
# First time setup
direnv allow

# Build the project
cargo build

# Run the application
cargo run

# Watch mode (auto-rebuild on changes)
cargo watch -x run

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Commit Guidelines

This project follows [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (white-space, formatting, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to build process or auxiliary tools

### Examples

```bash
# Feature commit
git commit -m "feat: add task creation in TUI"

# Bug fix with scope
git commit -m "fix(ui): correct column alignment issue"

# Breaking change
git commit -m "feat!: redesign storage format to SQLite"

# With body and footer
git commit -m "feat: add keyboard shortcuts

- Add 'n' for new task
- Add arrow keys for navigation
- Add 'd' for delete task

Closes #123"
```

## Testing Strategy

### Unit Tests
- Located alongside source code using `#[cfg(test)]` modules
- Test individual components and functions
- Focus on business logic and state management

### Integration Tests
- Located in `tests/` directory
- Test complete workflows and component interactions
- Verify TUI behavior and data persistence

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run tests in watch mode
cargo watch -x test
```

## Building and Running

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run development build
cargo run

# Run release build
./target/release/kanban-tui
```

## Dependencies

### Core Dependencies
- `ratatui`: Terminal UI framework
- `crossterm`: Cross-platform terminal manipulation
- `serde` + `serde_json`: Serialization for persistence
- `dirs`: Platform-specific directory paths for config storage

### Development Dependencies
- Standard Rust test framework
- `cargo-watch`: Development automation (provided by Nix)

## Features Roadmap

### Phase 1: Core Functionality (MVP)
- [x] Basic TUI with three columns (To Do, In Progress, Done)
- [x] Create and view tasks
- [x] Delete tasks
- [x] Move tasks between columns
- [x] Basic keyboard navigation (h/l or arrow keys, j/k for tasks)

### Phase 2: Enhanced Features
- [x] Persistent storage (JSON file)
- [ ] Edit task details
- [ ] Task descriptions and metadata
- [ ] Color coding and priorities

### Phase 3: Advanced Features
- [ ] Multiple boards
- [ ] Custom columns
- [ ] Task filtering and search
- [ ] Export/import functionality

## Key Decisions

1. **TUI Framework**: Using `ratatui` (fork of `tui-rs`) for active maintenance and features
2. **Terminal Backend**: `crossterm` for cross-platform compatibility
3. **State Management**: Centralized application state with event-driven updates
4. **Storage Format**: JSON for human-readable persistence
5. **Storage Location**: Platform-specific config directory (`~/.config/kanban-tui/board.json` on Linux, `~/Library/Application Support/kanban-tui/board.json` on macOS, `%APPDATA%\kanban-tui\board.json` on Windows)
6. **Auto-save**: Board automatically saves after every modification (create, delete, move)
7. **Auto-load**: Board automatically loads from storage on startup
8. **Development Environment**: Nix flakes for reproducibility across machines

## Storage Implementation

### Architecture
The storage module (`src/storage.rs`) provides persistent storage using JSON files:

- **Storage Location**: Uses `dirs` crate to get platform-specific config directory
  - Linux: `~/.config/kanban-tui/board.json`
  - macOS: `~/Library/Application Support/kanban-tui/board.json`
  - Windows: `%APPDATA%\kanban-tui\board.json`
- **Auto-save**: Automatically saves after create/delete/move operations
- **Auto-load**: Loads saved board on application startup
- **Error Handling**: Gracefully handles missing files (creates new board) and I/O errors

### Usage
```rust
use kanban_tui::storage::Storage;

// Create storage with default location
let storage = Storage::new()?;

// Save board
storage.save(&board)?;

// Load board (returns None if file doesn't exist)
let board = storage.load()?.unwrap_or_else(|| Board::new("Default".to_string()));

// Check if storage file exists
if storage.exists() {
    println!("Found existing board at: {:?}", storage.file_path());
}
```

## Notes for AI Assistants

- The codebase follows standard Rust conventions and idioms
- Tests should be written for all new features
- UI code should be separated from business logic for testability
- Use the provided Nix environment to ensure consistent tooling
- When adding dependencies, update both `Cargo.toml` and document in this file
- Storage tests use temporary file paths to avoid interfering with user data

# Kanban TUI

A terminal user interface (TUI) for managing Kanban boards, written in Rust.

## Features

- Interactive terminal-based Kanban board
- Three default columns: To Do, In Progress, Done
- Create, delete, and move tasks between columns
- Vim-style keyboard navigation (h/j/k/l)
- Persistent storage (auto-saves your board)
- Fast and lightweight
- Cross-platform support

## Development Setup

This project uses Nix flakes for reproducible development environments.

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- [direnv](https://direnv.net/) (recommended)

### Quick Start

```bash
# Clone the repository
git clone <your-repo-url>
cd kanban-tui

# Allow direnv (auto-loads Nix environment)
direnv allow

# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test
```

### Without direnv

```bash
# Enter the Nix development shell manually
nix develop

# Then use cargo as usual
cargo build
cargo run
cargo test
```

## Usage

### Keyboard Shortcuts

- `n` - Create a new task in the current column
- `e` - Edit the selected task title
- `h`/`l` or `←`/`→` - Navigate between columns
- `j`/`k` or `↑`/`↓` - Navigate between tasks
- `Shift+h`/`Shift+l` or `H`/`L` - Move selected task left/right
- `d` - Delete selected task
- `q` - Quit the application

### Data Storage

Your board is automatically saved after every change to:
- **Linux**: `~/.config/kanban-tui/board.json`
- **macOS**: `~/Library/Application Support/kanban-tui/board.json`
- **Windows**: `%APPDATA%\kanban-tui\board.json`

The board loads automatically when you start the application.

## Project Structure

```
kanban-tui/
├── src/
│   ├── main.rs          # Application entry point with TUI
│   ├── lib.rs           # Core library with business logic
│   └── storage.rs       # Persistent storage implementation
├── tests/               # Integration tests
├── flake.nix            # Nix development environment
├── .envrc               # direnv configuration
└── Cargo.toml           # Rust dependencies
```

## Development

```bash
# Auto-rebuild on file changes
cargo watch -x run

# Run tests in watch mode
cargo watch -x test

# Build optimized release binary
cargo build --release
```

## Testing

The project includes both unit tests (in `src/lib.rs` and `src/main.rs`) and integration tests (in `tests/`). Currently 62 tests covering all functionality.

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

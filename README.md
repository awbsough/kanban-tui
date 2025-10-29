# Kanban TUI

A terminal user interface (TUI) for managing Kanban boards, written in Rust.

## Features

- Interactive terminal-based Kanban board
- Three default columns: To Do, In Progress, Done
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

Press `q` to quit the application.

(More keybindings and features coming soon!)

## Project Structure

```
kanban-tui/
├── src/
│   ├── main.rs          # Application entry point with TUI
│   └── lib.rs           # Core library with business logic
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

The project includes both unit tests (in `src/lib.rs`) and integration tests (in `tests/`).

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

{
  description = "Kanban TUI - A terminal user interface for managing Kanban boards";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Development tools
            cargo-watch
            cargo-edit
            cargo-audit

            # For building
            pkg-config

            # Optional: useful utilities
            ripgrep
            fd
          ];

          shellHook = ''
            echo "Kanban TUI development environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build       - Build the project"
            echo "  cargo run         - Run the application"
            echo "  cargo test        - Run tests"
            echo "  cargo watch -x run - Auto-rebuild on changes"
            echo ""
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
        };

        # Package definition for building the application
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "kanban-tui";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          meta = with pkgs.lib; {
            description = "A terminal user interface for managing Kanban boards";
            homepage = "https://github.com/yourusername/kanban-tui";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}

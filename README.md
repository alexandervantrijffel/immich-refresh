# immich-refresh

A CLI tool for batch uploading photos to [Immich](https://immich.app/) with automatic album organization based on directory structure.

## What Does This Tool Do?

`immich-refresh` traverses a directory structure and automatically uploads photos to Immich, organizing them into albums based on the grandchild directory names. It's designed to work with a specific directory hierarchy where:

- **Base path**: The starting directory (e.g., `/photos`)
- **Child directories**: First level subdirectories (e.g., `/photos/2024`, `/photos/vacation`)
- **Grandchild directories**: Second level subdirectories that become album names (e.g., `/photos/2024/summer`, `/photos/vacation/beach`)

### Album Naming Logic

- By default, the **grandchild directory name** is used as the album name
- If a grandchild directory is named `"other"` (case-insensitive), the **parent (child) directory name** is used instead

**Example:**
```
/photos
├── 2024
│   ├── summer      → Album: "summer"
│   ├── winter      → Album: "winter"
│   └── other       → Album: "2024" (uses parent name)
└── vacation
    ├── beach       → Album: "beach"
    └── Other       → Album: "vacation" (case-insensitive)
```

### Features

- **Automatic album organization**: Albums are created based on directory structure
- **Sequential processing**: Directories are processed one at a time to avoid overwhelming the system
- **Error resilience**: Continues processing remaining directories even if one fails
- **Dry-run mode**: Preview what would be uploaded without actually executing commands
- **Logging**: Logs to both stdout and `/var/log/immich-refresh.log` (file logging disabled in dry-run mode)
- **Signal handling**: Gracefully handles SIGINT/SIGTERM by stopping current upload and exiting

## Prerequisites

- Rust 1.90 or later (edition 2021)
- [Immich CLI](https://immich.app/docs/features/command-line-interface) installed at `/usr/src/app/cli/bin/immich`
- Immich CLI must be authenticated to your Immich server

## Installation

### Using cargo-make (Recommended)

First, install `cargo-make`:

```bash
cargo install cargo-make
```

Then build the project:

```bash
# Development build
cargo make build

# Release build
cargo make build-release
```

### Using cargo directly

```bash
# Development build
cargo build

# Release build
cargo build --release
```

### Cross-compilation for x86_64

If you're developing on ARM64 (e.g., Apple Silicon) and need to deploy to x86_64 Linux:

```bash
# Install the target
cargo make install-cross-target

# Cross-compile
cargo make cross-compile
```

The binary will be available at `target/x86_64-unknown-linux-gnu/release/immich-refresh`.

## Usage

### Basic Usage

```bash
immich-refresh <path>
```

**Example:**
```bash
immich-refresh /mnt/photos
```

This will:
1. Traverse `/mnt/photos`
2. Find all grandchild directories
3. Upload photos from each grandchild directory to Immich
4. Create albums based on the directory names

### Dry-Run Mode

Preview what would be executed without actually uploading:

```bash
immich-refresh <path> --dry-run
```

**Example:**
```bash
immich-refresh /mnt/photos --dry-run
```

### Using cargo-make

```bash
# Normal run
cargo make run -- /mnt/photos

# Dry-run
cargo make run-dry-run -- /mnt/photos
```

## Development

### Running Tests

```bash
# Run all tests
cargo make test

# Run tests with verbose output
cargo make test-verbose

# Using cargo directly
cargo test
```

### Code Quality

```bash
# Format code
cargo make fmt

# Check formatting (CI-friendly)
cargo make fmt-check

# Run clippy linter
cargo make clippy

# Check without building
cargo make check
```

### Development Workflow

Run the complete development workflow (format, check, test, build):

```bash
cargo make all
```

### CI Workflow

Run CI checks (format check, clippy, test, build):

```bash
cargo make ci
```

## Logging

The tool uses structured logging with the `tracing` crate:

- **Stdout**: Always logs to stdout
- **File**: Logs to `/var/log/immich-refresh.log` (append mode) in normal mode
- **Dry-run**: File logging is disabled in dry-run mode

Log location is printed at startup:
```
INFO Logging to stdout and /var/log/immich-refresh.log
```

### Setting Log Level

Use the `RUST_LOG` environment variable to control log verbosity:

```bash
# Debug level
RUST_LOG=debug immich-refresh /mnt/photos

# Trace level (very verbose)
RUST_LOG=trace immich-refresh /mnt/photos
```

## Project Structure

```
immich-refresh/
├── Cargo.toml              # Workspace manifest
├── Makefile.toml           # cargo-make task definitions
├── README.md               # This file
├── CLAUDE.md               # Development guide for Claude Code
└── crates/
    └── refresh-cli/        # Main CLI crate
        ├── Cargo.toml
        └── src/
            ├── main.rs           # Entry point and argument parsing
            ├── prelude.rs        # Common imports
            ├── execute.rs        # Command execution trait and implementation
            ├── traverse.rs       # Directory traversal logic
            └── tracing_config.rs # Logging configuration
```

## How It Works

1. **Parse arguments**: Validates command line arguments (path and optional --dry-run flag)
2. **Configure logging**: Sets up tracing to stdout and optionally to log file
3. **Traverse directories**:
   - Iterates through child directories of the base path
   - For each child, iterates through grandchild directories
   - Determines album name based on grandchild name (or parent if "other")
4. **Execute uploads**: For each grandchild directory, runs:
   ```bash
   /usr/src/app/cli/bin/immich upload -H -r -c 24 -A <album_name> <path>/*
   ```
5. **Handle errors**: Logs errors and continues processing remaining directories

## Error Handling

- **Path doesn't exist**: Exits with error
- **Path is not a directory**: Exits with error
- **Upload fails**: Logs error and continues with next directory
- **Authentication error**: Immich CLI error is displayed to user
- **Signal received (Ctrl+C)**: Kills current upload and exits with code 130

## Timeout

Each upload command has a 3-hour timeout. If an upload doesn't complete within this time, it's terminated.

## License

This project is licensed under the terms specified in the LICENSE file.

## Contributing

1. Format your code: `cargo make fmt`
2. Run tests: `cargo make test`
3. Run linter: `cargo make clippy`
4. Ensure CI passes: `cargo make ci`

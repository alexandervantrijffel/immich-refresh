# immich-refresh

A Rust CLI tool for batch uploading photos to [Immich](https://immich.app/) with intelligent album organization based on directory structure.

## Overview

`immich-refresh` automates the process of uploading large photo collections to Immich by traversing a two-level directory structure and creating albums based on folder names. Perfect for organizing photos from multiple events, years, or categories without manual album creation.

### How It Works

The tool operates on a **grandchild directory pattern**:

- **Base path**: Your photo collection root (e.g., `/mnt/photos`)
- **Child directories**: Top-level categories (e.g., `/mnt/photos/2024`, `/mnt/photos/vacation`)
- **Grandchild directories**: Specific albums (e.g., `/mnt/photos/2024/summer`, `/mnt/photos/vacation/beach`)

Each grandchild directory becomes an Immich album with all its photos uploaded.

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

### Key Features

- **Automatic album organization** - Albums created from directory names
- **Sequential processing** - One directory at a time to prevent system overload
- **Error resilient** - Continues on failures, logs errors for review
- **Dry-run mode** - Preview operations without uploading
- **Comprehensive logging** - Dual output to stdout and `~/.local/state/immich-refresh/run.log`
- **Signal handling** - Graceful shutdown on Ctrl+C (SIGINT/SIGTERM)
- **Smart naming** - Handles "other" directories by using parent names

## Prerequisites

- Rust 1.90 or later (edition 2021)
- [Immich CLI](https://immich.app/docs/features/command-line-interface) installed and available in PATH as `immich`
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

If you're developing on ARM64 (e.g., Apple Silicon, ARM Linux) and need to deploy to x86_64 Linux:

```bash
# Install the x86_64 target
cargo make install-target

# Build for x86_64 (compile only, no tests)
cargo make build-x86_64
```

The binary will be available at `target/x86_64-unknown-linux-gnu/release/immich-refresh`.

**Note:** Tests should be run on your native architecture:
```bash
cargo test
```

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
- **File**: Logs to `~/.local/state/immich-refresh/run.log` (append mode) in normal mode
- **Dry-run**: File logging is disabled in dry-run mode

The log directory is automatically created if it doesn't exist. Log location is printed at startup:
```
INFO Logging to stdout and /home/user/.local/state/immich-refresh/run.log
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
   immich upload -H -r -c 24 -A <album_name> <path>/*
   ```
5. **Handle errors**: Logs errors and continues processing remaining directories

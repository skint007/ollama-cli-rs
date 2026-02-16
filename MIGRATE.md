# Ollama CLI Tool - Rust Tech Stack

## CLI & Argument Parsing
- **`clap`** (v4+) - Industry-standard command-line argument parser. Use the derive API for clean, declarative definitions of commands, subcommands, flags, and arguments with automatic help generation.

## TUI Components
- **`ratatui`** - Modern terminal UI framework for building rich, interactive terminal interfaces. Provides widgets (lists, tables, charts), flexible layouts, and event-driven architecture. Successor to the original `tui-rs`.

- **`crossterm`** - Cross-platform terminal manipulation library. Handles terminal capabilities like colors, cursor control, keyboard/mouse input, and works seamlessly across Windows, macOS, and Linux. Required backend for `ratatui`.

## Interactive Elements
- **`inquire`** - Modern library for interactive CLI prompts including text input, confirmations, selections, multi-select menus, and password fields. Features built-in validation and excellent UX.

- **`indicatif`** - Progress bars and spinners for CLI operations. Supports multiple concurrent progress indicators, customizable styles, and integrates well with async operations for showing API call progress.

## HTTP & API Communication
- **`reqwest`** - Ergonomic async HTTP client for communicating with the Ollama API. Use with `json` feature for automatic serialization/deserialization of request/response bodies.

## Async Runtime
- **`tokio`** - Async runtime required for handling async HTTP requests to Ollama. Provides task scheduling, async I/O, timers, and other async primitives.

## Serialization
- **`serde`** - Serialization framework for converting between Rust structs and data formats.

- **`serde_json`** - JSON implementation for `serde`. Essential for parsing Ollama API responses and building request payloads.

## Error Handling
- **`anyhow`** - Simplified error handling with context. Provides `Result<T>` type alias and easy error chaining with `.context()` for better error messages.

## Optional Additions
- **`config`** - Configuration file management if you need to support `.toml`, `.yaml`, or `.json` config files for storing Ollama instance URLs, default models, etc.

- **`directories`** - Cross-platform path helpers for finding standard directories (config, cache, data) to store user preferences and history.
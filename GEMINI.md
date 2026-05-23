# Gemini Project Instructions for PVM (PHP Version Manager)

Welcome to the PVM Rust rewrite project. This file provides context, coding guidelines, and references for Gemini agents working in this codebase.

## Tech Stack & Architecture

* **Language**: Rust (Edition 2024)
* **CLI Parser**: `clap` with `derive` macros (nested subcommand design).
* **Database**: SQLite accessed via `rusqlite` (configured with `bundled` feature for portability).
* **Network & Scraping**: `reqwest` (blocking client) and `scraper` for fetching/parsing release pages.
* **Extraction**: `zip` crate for archive extraction.

## Project Structure

* [main.rs](file:///D:/me/PVM/src/main.rs): Bootstraps the application context, resolves paths relative to the executable location, parses CLI inputs using `clap`, and routes commands.
* [lib.rs](file:///D:/me/PVM/src/lib.rs): Declares submodules and suppresses uppercase warning for the crate name (`#![allow(non_snake_case)]`).
* [db.rs](file:///D:/me/PVM/src/db.rs): Handles Sqlite table migrations and row persistence. It enforces schema compatibility with the previous C# EF Core version.
* [commands.rs](file:///D:/me/PVM/src/commands.rs): Encapsulates the logic of each subcommand (`add`, `list`, `use`, `ini`, `ext`, `ext-enable`, `install`).
* [helpers.rs](file:///D:/me/PVM/src/helpers.rs): Holds file, network, and extraction utility functions.
* [tests.rs](file:///D:/me/PVM/tests/tests.rs): Sandboxed integration tests.

---

## Coding Standards

1. **Path Management**: Always resolve database files and PHP target directory modifications relative to the executable parent directory using `std::env::current_exe()` context (`base_dir`). Avoid referencing standard user folders or the shell's active working directory directly, unless running under sandboxed testing.
2. **Naming Conventions**: Match EF Core database table naming casing (`PhpVersions`, `InstallUrls`) and column names (`Id`, `IsCurrent`, `Path`, `Version`) to support legacy installations without needing data migration.
3. **Error Handling**: Use `Result` returning standard `Box<dyn std::error::Error>` for commands and library entrypoints, promoting bubble-up error propagation and user-friendly error messages.
4. **Windows Compatibility**: Spawning GUI processes (such as Notepad or Explorer) must be conditionally compiled under `#[cfg(target_os = "windows")]` blocks and print descriptive path fallbacks on other platforms.

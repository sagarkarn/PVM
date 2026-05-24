---
name: rust-dev
description: Commands and procedures for compiling, testing, checking, formatting, and building the Rust project.
---

# Rust Development Skill

This skill assists in running common Cargo commands to maintain the quality, correctness, and formatting of the Rust PVM project.

## Commands

### Building
To compile the application in debug mode:
```powershell
cargo build
```

To compile the application in release mode (production ready binary):
```powershell
cargo build --release
```

### Running Tests
To run all tests (unit and integration tests):
```powershell
cargo test
```

To run a specific test:
```powershell
cargo test <test_name>
```

### Code Formatting
To format the code according to the Rust standards:
```powershell
cargo fmt
```

### Code Linting
To check code using clippy:
```powershell
cargo clippy
```

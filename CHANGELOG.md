# Changelog

All notable changes to this project will be documented in this file.

## 2.1.4 - 2026-05-27

### Added
- Added a new `version` subcommand to print the application's version.
- Configured `-v` and `--version` flags on the root CLI command to print the version details and exit.
- Added a new integration test `test_version_command` to verify version output correctness.
- Added the raw executable (`pvm-v<version>-windows-x64.exe`) as a release asset in the GitHub release workflow.

### Changed
- Centralized the application version string into a unified `PVM_VERSION` constant.
- Updated the `self-update` and automatic daily update checking features to use the centralized `PVM_VERSION` instead of raw package version macros.
- Modified the main CLI parsing loop to allow running without subcommands (which now displays the help page).

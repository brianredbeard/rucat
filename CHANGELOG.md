# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-08-19

This is the initial tagged release of `rucat`.

### Features

- **Multiple Output Formats**: Supports `ansi`, `utf8`, `markdown`, `ascii`, `xml`, `json`, and `pretty` (syntax highlighting) formats.
- **Line Numbering**: Prepend line numbers to output with the `-n`/`--numbers` flag.
- **Clipboard Support**: Copy output directly to the system clipboard with the `-c`/`--copy` flag.
- **Flexible Input**: Process files, directories, `stdin`, and NUL-separated file lists from commands like `find`.
- **Path Manipulation**: Use `--strip` to remove leading path components from file headers.
- **Configuration**: Set default options via a `config.toml` file in the user's config directory.

### Bug Fixes

- Resolved Rust 2024 Edition compatibility lints (`ref mut` bindings).
- Silenced unavoidable duplicate dependency warnings flagged by `clippy::cargo`.
- Corrected conditional compilation guards for the clipboard feature to eliminate unused variable warnings.
- Fixed logic in the release workflow to ensure smoke tests run correctly on macOS ARM builds.

### Build

- Pinned the Rust toolchain to `stable` using `rust-toolchain.toml` for consistent builds.
- Regenerated `Cargo.lock` to resolve dependency graph corruption.
- Configured `cargo deny` to allow an unmaintained transitive dependency (`yaml-rust`) and unavoidable duplicate crate versions.
- Removed invalid `.clippy.toml` configuration file.

### Continuous Integration

- Updated `check.sh` to accurately simulate the CI environment by isolating `CARGO_HOME`.
- Ensured all build and test commands in the release workflow consistently use the correct target architecture.
- Standardized shell execution to `bash` across all operating systems in CI workflows for improved reliability.

### Refactor

- Improved function signatures in the main binary to resolve `clippy::too_many_arguments` and `clippy::needless_pass_by_value` lints.

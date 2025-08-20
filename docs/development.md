# Development Guide

This document provides a guide for developers working on the `rucat` project. It
covers the development workflow, the structure of the `Makefile`, and the
Continuous Integration (CI) process.

## Using the Makefile for Development and Validation

The `Makefile` is the cornerstone of our development and CI process. It provides
a set of consistent, cross-platform commands for building, testing, and
validating the project. By using the `Makefile`, you can run the exact same
checks locally that our CI server runs, ensuring that your contributions will
pass automated checks before you even create a pull request.

### Main Development Targets

These targets are for your day-to-day development workflow.

- `make build`: Compiles the project in debug mode with all optimizations
  disabled. This is the fastest way to check for compilation errors.
- `make release`: Compiles the project in release mode with optimizations
  enabled, creating a production-ready binary in `target/release/`.
- `make run ARGS="..."`: Runs the project, passing any arguments you provide via
  the `ARGS` variable.
  - Example: `make run ARGS="-f ansi README.md"`
- `make test`: Runs the entire test suite.
- `make check`: Checks the source code for compilation errors without producing
  a final binary. It's faster than a full `make build`.

### Linting and Validation Targets

These targets help ensure code quality, consistency, and correctness.

- `make lint`: **(Recommended)** This is the primary target for developers to
  run before committing code. It performs a fast set of essential checks:
  - `make fmt-check`: Verifies that the code is formatted according to the
    project's style guidelines (`rustfmt`).
  - `make clippy`: Runs the Rust linter to catch common mistakes and unidiomatic
    code.
- `make fmt`: Automatically formats all code in the project. Run this if
  `make fmt-check` fails.
- `make clippy-pedantic`: Runs a much stricter version of the linter with more
  opinionated checks. Useful for catching subtle issues.
- `make deny`: Checks for crate policy violations, such as dependencies with
  incompatible licenses or known security vulnerabilities.

### CI Emulation Targets

These targets are designed to precisely replicate the jobs run in our GitHub
Actions CI pipeline.

- `make ci`: **(Comprehensive Check)** This is the master command to run the
  full suite of CI validation checks locally. It aggregates the linting,
  security, and core testing jobs. Use this for a final, thorough check of your
  changes.
  - `make ci-lint`: Runs all checks from the CI "Linting" stage, including
    formatting (`fmt-check`), linting (`clippy`), documentation checks
    (`doc-check`), and a check to ensure no files were modified by the process
    (`check-dirty`).
  - `make ci-test`: Runs the test suite under various feature configurations,
    mirroring the CI test matrix.
  - `make ci-security`: Runs the security audit and dependency policy checks.
    Unlike the stricter developer-facing targets, these `*-ci` versions are
    designed to generate report files (`audit-report.json`, `deny-report.json`)
    and print warnings without immediately failing. This matches the CI's
    behavior of "warn and report."

### Other Utility Targets

- `make generate-assets`: **(Important)** Run this target whenever you make
  changes to the command-line interface in `src/cli.rs`. It regenerates the man
  page and shell completion files in the `assets/` directory.
- `make coverage`: Generates a code coverage report. The results are saved to
  `lcov.info`. To view a detailed HTML report, run
  `cargo llvm-cov report --html --output-dir coverage-html` after this command.
- `make bench`: Runs the performance benchmark suite.
- `make cross-build TARGET=...`: Cross-compiles the project for a different
  architecture.
  - Example: `make cross-build TARGET=aarch64-unknown-linux-gnu`
- `make clean`: Removes all build artifacts.

## Continuous Integration (CI) Workflow Overview

Our CI pipeline is defined in `.github/workflows/ci-make.yaml` and is triggered
on every push and pull request to the `main` and `develop` branches. It is
designed to be highly parallel to provide fast feedback. The workflow consists
of several jobs that use the `Makefile` targets described above.

The CI process is broken down into the following stages:

1. **Initial Parallel Checks**:

   - **Linting**: A set of jobs run concurrently to check for code quality and
     correctness (`Format Check`, `Clippy`, `Doc Check`).
   - **Security**: Two jobs run in parallel to check for vulnerabilities
     (`Cargo Audit`) and policy violations (`Cargo Deny`). These jobs generate
     reports that can be inspected.
   - **Dirty Check**: A job that runs after the format check to ensure that no
     source files were modified during the linting process.

1. **Core Testing**:

   - Once all linting jobs succeed, the main `test` matrix is triggered. This
     job builds and runs the test suite across multiple configurations to ensure
     broad compatibility:
     - Different Operating Systems (Linux, Windows, macOS)
     - Different Rust Versions (Stable, Beta, Nightly, and MSRV)
     - Different Feature Sets (`default`, `minimal`, `all`)

1. **Post-Test Analysis & Artifacts**:

   - After the test matrix completes successfully, several jobs run in parallel:
     - `Code Coverage`: Calculates the test coverage of the codebase and uploads
       the report to Codecov.
     - `Performance Benchmarks`: Runs the benchmark suite (only on pushes to
       `main`, not on PRs).
     - `Cross-Compile`: Builds release binaries for several common Linux targets
       to ensure they compile correctly.

1. **Final Summary**:

   - A final `CI Summary` job runs at the end, regardless of whether the
     previous jobs succeeded or failed. It collects the status of all other jobs
     and provides a high-level overview of the entire pipeline's result, making
     it easy to see which part of the process failed.

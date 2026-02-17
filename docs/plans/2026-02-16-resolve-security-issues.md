# Resolve Security Issues Implementation Plan

Created: 2026-02-16
Status: VERIFIED
Approved: Yes
Iterations: 0
Worktree: Yes

> **Status Lifecycle:** PENDING → COMPLETE → VERIFIED
> **Iterations:** Tracks implement→verify cycles (incremented by verify phase)
>
> - PENDING: Initial state, awaiting implementation
> - COMPLETE: All tasks implemented
> - VERIFIED: All checks passed
>
> **Approval Gate:** Implementation CANNOT proceed until `Approved: Yes`
> **Worktree:** Set at plan creation (from dispatcher). `Yes` uses git worktree isolation; `No` works directly on current branch (default)

## Summary

**Goal:** Fix four security vulnerabilities identified during security review of the `feature/refactor-metadata-and-cli` branch: XML path injection, panic on zero-width binary formatting, XML comment breakout with metadata, and test env var leak into production code.

**Architecture:** Targeted fixes to four existing files with no structural changes. Each fix is independent and adds corresponding unit/integration tests.

**Tech Stack:** Rust, clap (CLI argument validation), existing test infrastructure (assert_cmd, predicates, tempfile)

## Scope

### In Scope

- Escape `path.display()` in XML attribute output (`src/formatters/xml.rs`)
- Validate `--hex-width` and `--base64-width` to reject zero values (`src/cli.rs`)
- Sanitize metadata content in XML comments to prevent `-->` breakout (`src/formatters/xml.rs`)
- Gate `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` env var behind `#[cfg(test)]` (`src/clipboard.rs`)

### Out of Scope

- Windows `unsafe` FFI blocks (standard Win32 patterns, no actionable fix)
- `exacl` dependency platform scope (build issue, not security)
- `to_string_lossy` on non-UTF8 paths (cosmetic, not exploitable)

## Prerequisites

- Rust toolchain with `cargo test` working
- `xattr` support on the test filesystem (for binary display tests)

## Context for Implementer

- **Patterns to follow:** Existing XML escaping function `esc()` at `src/formatters/xml.rs:26-32` — reuse for path escaping
- **Conventions:** Tests use `assert_cmd::Command` for CLI integration tests, `predicates` crate for assertions. Unit tests for formatters use `capture_with_path()` helper in `tests/formatter_units.rs:25-41`
- **Key files:**
  - `src/formatters/xml.rs` — XML formatter with `esc()` function and `Formatter::write()` impl
  - `src/binary_display.rs` — `BinaryDataFormatter` with `format_hex_dump()` and `format_base64()` methods
  - `src/cli.rs` — CLI argument definitions using clap derive macros
  - `src/clipboard.rs` — `ClipboardProvider::auto_detect()` with env var override
  - `tests/formatter_units.rs` — unit tests for all formatters with `capture_with_path()` helper
  - `tests/cli_binary_display.rs` — integration tests for binary display CLI flags
  - `tests/clipboard.rs` — clipboard integration tests using `assert_cmd`
- **Gotchas:** Clipboard tests use `#[cfg(feature = "clipboard")]` gating. Some clipboard auto-detect tests only run on `#[cfg(all(unix, not(target_os = "macos")))]`. The `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` env var is used by integration tests (e.g., `tests/clipboard.rs:100-120`) that set it via `.env()` on `Command`, so removing it from production code requires those tests to still work via the `--clipboard-provider-for-test` CLI flag.

## Progress Tracking

**MANDATORY: Update this checklist as tasks complete. Change `[ ]` to `[x]`.**

- [x] Task 1: Fix XML path injection
- [x] Task 2: Validate hex_width and base64_width to prevent zero-value panics
- [x] Task 3: Sanitize XML comment content to prevent --> breakout
- [x] Task 4: Gate test env var behind #[cfg(test)]

**Total Tasks:** 4 | **Completed:** 4 | **Remaining:** 0

## Implementation Tasks

### Task 1: Fix XML path injection

**Objective:** Escape `path.display()` output using the existing `esc()` function when interpolating into XML `path="..."` attributes, preventing XML injection via crafted filenames.

**Dependencies:** None

**Files:**

- Modify: `src/formatters/xml.rs` (lines 49, 57-58)
- Test: `tests/formatter_units.rs`

**Key Decisions / Notes:**

- The `esc()` function at `src/formatters/xml.rs:26-32` already handles `&`, `<`, `>`, `"`, and `'` — all characters needed for safe XML attribute values. It's a private function in the same module, so no visibility change needed.
- Apply `esc(&path.display().to_string())` on both line 49 (line-numbered mode) and line 57 (plain mode)
- Existing test `xml_escaping` at `tests/formatter_units.rs:107-113` tests content escaping; add a parallel test for path escaping
- Note: XML comments (`<!-- -->`) can contain `<`, `>`, `&`, `'` without escaping (not parsed). Only `--` is forbidden in comments. So path attributes need full `esc()` escaping (parsed context), but comment content only needs `--` sanitization (Task 3).

**Definition of Done:**

- [x] `path.display()` is escaped via `esc()` on both XML output paths (lines 49 and 57)
- [x] Test with filename containing `"`, `<`, `>`, `&` characters verifies proper escaping in XML attribute
- [x] All existing formatter tests pass

**Verify:**

- `cargo test -q` — full test suite passes (path escaping could affect any test asserting on XML output)
- `cargo test xml -q` — XML-specific tests pass

### Task 2: Validate hex_width and base64_width to prevent zero-value panics

**Objective:** Prevent `chunks(0)` panic by adding clap validation to reject zero values for `--hex-width` and `--base64-width` CLI arguments, and clamping config values.

**Dependencies:** None

**Files:**

- Modify: `src/cli.rs` (lines 51-52, 55-56)
- Modify: `src/main.rs` (lines 249-250)
- Test: `tests/cli_binary_display.rs` (CLI integration test for `--hex-width 0`)
- Test: `tests/cli_config.rs` (integration test for config file with `hex_width = 0`)

**Key Decisions / Notes:**

- Use clap's `value_parser!(usize).range(1..)` on `--hex-width` and `--base64-width` to reject zero at parse time. This gives the user a clear error message from clap rather than a panic.
- Also clamp config file values in `src/main.rs` with `.max(1)` after `unwrap_or(16)` / `unwrap_or(76)` to handle zero in `config.toml`
- The `format_hex_dump` method at `src/binary_display.rs:100` calls `self.data.chunks(bytes_per_line)` — with `bytes_per_line=0` this panics. Similarly `format_base64` at line 159 calls `.chunks(width)`. The Intel HEX formatter at line 184 uses `self.data.chunks(16)` (hardcoded), which is safe.
- Config file test: use the existing `prepare_config` helper in `tests/cli_config.rs` to write `hex_width = 0` and verify the program runs without panicking

**Definition of Done:**

CLI validation (clap):
- [x] `--hex-width 0` produces a clap error, not a panic
- [x] `--base64-width 0` produces a clap error, not a panic
- [x] CLI integration test confirms `--hex-width 0` exits with failure and error message

Config file validation (main.rs):
- [x] Config file `hex_width = 0` is clamped to 1
- [x] Config file `base64_width = 0` is clamped to 1
- [x] Integration test with config file containing `hex_width = 0` runs without panicking

- [x] All existing binary display tests pass

**Verify:**

- `cargo test --test cli_binary_display -q` — all binary display tests pass
- `cargo test --test cli_config -q` — all config tests pass (including zero-width config test)
- `cargo test --test formatter_units -q` — all formatter tests pass

### Task 3: Sanitize XML comment content to prevent --> breakout

**Objective:** When metadata is rendered inside an XML comment (`<!-- ... -->`), sanitize the content to prevent `-->` sequences from breaking out of the comment.

**Dependencies:** None

**Files:**

- Modify: `src/formatters/xml.rs` (line 45)
- Test: `tests/formatter_units.rs`

**Key Decisions / Notes:**

- The XML spec prohibits `--` inside comments. The standard sanitization is to replace `--` with `- -` (insert a space). This also prevents `-->` since it contains `--`.
- Apply this sanitization to `formatted_meta` before writing the comment in `xml.rs:45`
- The sanitization should be a local helper function in `xml.rs` (e.g., `fn sanitize_xml_comment(s: &str) -> String`) that replaces `--` with `- -`
- This is conservative — it prevents both `--` (which is invalid in XML comments per spec) and `-->` (which breaks out)
- Triple hyphens `---` are also handled: each `--` pair is replaced, so `---` becomes `- --` on first pass, then the remaining `--` becomes `- -`, giving `- - -`. Use a loop or `.replace("--", "- -")` applied until stable.

**Definition of Done:**

- [x] Metadata containing `-->` does not break out of the XML comment
- [x] Metadata containing `--` (without `>`) is also sanitized (per XML spec)
- [x] Metadata containing `---` (triple hyphens) is sanitized to `- - -`
- [x] Test with metadata path/xattr containing `-->` verifies sanitized output
- [x] All existing formatter tests pass

**Verify:**

- `cargo test --test formatter_units -q` — all formatter tests pass

### Task 4: Gate test env var behind #[cfg(test)]

**Objective:** Move the `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` environment variable check out of the production `auto_detect()` code path, so it's only reachable in test builds.

**Dependencies:** None

**Files:**

- Modify: `src/clipboard.rs` (lines 33-42)
- Test: `tests/clipboard.rs`

**Key Decisions / Notes:**

- The env var `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` is currently checked at the top of `auto_detect()` (line 35). In production, an attacker who can set env vars could force the clipboard provider.
- Wrap the env var block in `#[cfg(test)]` so it's compiled out in release builds
- **Verified:** `grep` confirms `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` appears only in `src/clipboard.rs:35` — no integration test references it. Integration tests control detection via `.env("TMUX", ...)`, `.env("TERM", "xterm-kitty")`, etc.
- **`#[cfg(test)]` behavior:** In Rust, `#[cfg(test)]` in library code (`src/clipboard.rs`) is compiled out when building the library for integration tests too (integration tests link against the library compiled without test config). Since no integration test uses this env var, this is safe.
- The `--clipboard-provider-for-test` CLI flag (handled in `src/main.rs`) is separate and unaffected — it bypasses `auto_detect()` entirely

**Definition of Done:**

- [x] The env var check block (lines 33-42 in `src/clipboard.rs`) is wrapped in `#[cfg(test)]`, making it compile-time excluded from release builds
- [x] All existing clipboard tests pass
- [x] Auto-detection integration tests (`auto_detect_tmux_selects_osc52`, etc.) still pass

**Verify:**

- `cargo test --test clipboard -q` — all clipboard tests pass
- `cargo test clipboard -q` — all clipboard-related tests pass

## Testing Strategy

- **Unit tests:** Path escaping in XML formatter, XML comment sanitization, zero-width handling (formatter_units.rs)
- **Integration tests:** CLI rejection of `--hex-width 0` (cli_binary_display.rs), clipboard auto-detect behavior (clipboard.rs)
- **Manual verification:** `cargo run -- --hex-width 0 somefile` confirms clap error

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| XML comment sanitization changes output format | Low | Low | Only affects content inside `<!-- -->` blocks; no consumers parse these |
| `#[cfg(test)]` gating breaks test that uses env var directly | Low | Medium | The `RUCAT_CLIPBOARD_PROVIDER_FOR_TEST` check is wrapped in `#[cfg(test)]` at `src/clipboard.rs` line 33, ensuring the code path is compiled out of release builds. Integration tests use `.env()` on `Command` and the `--clipboard-provider-for-test` CLI flag, neither of which depend on this env var in production code. |
| clap `value_parser` range validation changes error format | Low | Low | Clap's `value_parser` range validation (`1..`) provides clear error messages including the rejected value and valid range, maintaining compatibility with existing error handling |

## Open Questions

None — all issues are well-defined with clear fixes.

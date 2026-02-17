// This file is part of rucat.
//
// rucat is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rucat is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rucat.  If not, see <https://www.gnu.org/licenses/>.
//
// Copyright (C) 2024 Brian 'redbeard' Harrington
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn prepare_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = File::create(&p).unwrap();
    write!(f, "{body}").unwrap();
    p
}

fn prepare_config(config_dir: &std::path::Path, content: &str) {
    // This helper will create the config file at the standard XDG path inside
    // the provided `config_dir`, which will be used as a fake HOME directory.
    let mut rucat_config_dir = config_dir.to_path_buf();
    if cfg!(target_os = "macos") {
        rucat_config_dir.push("Library/Application Support/rucat");
    } else {
        rucat_config_dir.push(".config/rucat");
    }
    fs::create_dir_all(&rucat_config_dir).unwrap();
    let config_path = rucat_config_dir.join("config.toml");
    fs::write(config_path, content).unwrap();
}

/// Apply environment variables needed so the subprocess finds our fake config.
/// On Linux/non-macOS, XDG_CONFIG_HOME must also be set so it overrides any
/// system-level XDG_CONFIG_HOME (e.g., on CI runners).
fn apply_config_env(cmd: &mut Command, home_dir: &std::path::Path) {
    cmd.env("HOME", home_dir);
    if !cfg!(target_os = "macos") {
        cmd.env("XDG_CONFIG_HOME", home_dir.join(".config"));
    }
}

#[test]
fn config_default_format() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let file = prepare_file(dir.path(), "a.txt", "hello");
    prepare_config(home_dir.path(), "format = \"ascii\"\nstrip = 99");

    let mut cmd = Command::cargo_bin("rucat").unwrap();
    apply_config_env(&mut cmd, home_dir.path());
    cmd.arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("=== a.txt ==="));
}

#[test]
fn config_other_defaults() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let relative_path = "foo/a.txt";
    prepare_file(dir.path(), relative_path, "hello\nworld");
    // Set numbers=true, strip=1, and a large width
    prepare_config(
        home_dir.path(),
        "numbers = true\nstrip = 1\nansi_width = 120",
    );

    let mut cmd = Command::cargo_bin("rucat").unwrap();
    apply_config_env(&mut cmd, home_dir.path());
    cmd.current_dir(dir.path()) // Run from within the temp dir
        // The config sets numbers=true and strip=1. We override format on the CLI.
        .arg("-f")
        .arg("ansi") // Force ansi to check width
        .arg(relative_path) // Use the relative path
        .assert()
        .success()
        // Check for line numbers
        .stdout(predicate::str::contains("1 │ hello"))
        // Check that path is stripped
        .stdout(predicate::str::contains("a.txt").and(predicate::str::contains("foo").not()));
}

#[test]
fn config_cli_overrides() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let file = prepare_file(dir.path(), "a.txt", "hello");
    // Config says ascii, but CLI will say markdown
    prepare_config(home_dir.path(), r#"format = "ascii""#);

    let mut cmd = Command::cargo_bin("rucat").unwrap();
    apply_config_env(&mut cmd, home_dir.path());
    cmd.arg("-f")
        .arg("markdown")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("```").and(predicate::str::contains("===").not()));
}

#[test]
fn config_hex_width_zero_is_clamped() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let file = prepare_file(dir.path(), "a.txt", "test content");
    // Config with hex_width = 0 should be clamped to 1, not panic
    prepare_config(home_dir.path(), "hex_width = 0");

    let mut cmd = Command::cargo_bin("rucat").unwrap();
    apply_config_env(&mut cmd, home_dir.path());
    cmd.arg(&file).assert().success(); // Should not panic
}

#[test]
fn config_base64_width_zero_is_clamped() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let file = prepare_file(dir.path(), "a.txt", "test content");
    // Config with base64_width = 0 should be clamped to 1, not panic
    prepare_config(home_dir.path(), "base64_width = 0");

    let mut cmd = Command::cargo_bin("rucat").unwrap();
    apply_config_env(&mut cmd, home_dir.path());
    cmd.arg(&file).assert().success(); // Should not panic
}

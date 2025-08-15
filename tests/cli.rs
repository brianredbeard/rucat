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
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn prepare_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap(); // ensure nested dirs exist
    }
    let mut f = File::create(&p).unwrap();
    write!(f, "{body}").unwrap();
    p
}

#[test]
fn cli_ascii_numbers() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello\nworld\n");
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii", "-n"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 | hello").and(predicate::str::contains("2 | world")));
}

#[test]
fn cli_directory_recursion() {
    let dir = tempdir().unwrap();
    prepare_file(dir.path(), "x.rs", "fn main(){}");
    prepare_file(dir.path(), "y.c", "int main(){}");
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "markdown"])
        .arg(dir.path()) // pass directory, not files
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File:") // header appears twice
                .count(2),
        );
}

#[test]
fn cli_bad_file_reports_error() {
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["no_such_file.txt"])
        .assert()
        .stderr(predicate::str::contains("Error reading"))
        .success(); // program keeps going, exits 0
}

#[test]
fn cli_invalid_format_fails() {
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["--format", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}
#[test]
fn cli_strip_components() {
    let dir = tempdir().unwrap();
    let p = prepare_file(dir.path(), "foo/bar/baz.h", "x"); // create sub-dirs
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii", "--strip", "2"])
        .arg(&p)
        .assert()
        .success()
        // after --strip 2 the printed header must end with “baz.h”
        .stdout(predicate::str::contains("baz.h"));
}

#[test]
fn cli_pretty_default_by_extension() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.rs", "fn main() {}");
    Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")); // Should be highlighted
}

#[test]
fn cli_pretty_syntax_flag_overrides_extension() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    // A .txt file with rust content
    let file = prepare_file(dir.path(), "a.txt", "fn main() {}");
    // Get output when highlighted as Rust (from flag)
    let out_rust = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty", "--pretty-syntax", "rust"])
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    // Get output when highlighted as plain text (from .txt extension)
    let out_plain = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    // The two highlighted versions should be different.
    assert_ne!(
        out_rust, out_plain,
        "Highlighting with flag should differ from highlighting by extension"
    );
}

#[test]
fn cli_pretty_modeline_overrides_extension() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    // A .txt file with rust content and a modeline
    let file = prepare_file(dir.path(), "b.txt", "fn main() {}\n// vim: ft=rust");
    Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")); // Should be highlighted as Rust
}

#[test]
fn cli_pretty_flag_overrides_modeline() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    // A file with shell content, a modeline for TOML, but overridden by flag for shell
    let file = prepare_file(dir.path(), "c.txt", "echo 'hello'\n# vim: ft=toml");

    // Get output when highlighted as shell (from flag)
    let out_sh = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty", "--pretty-syntax", "sh"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")) // Should be highlighted
        .get_output()
        .stdout
        .clone();

    // Get output when highlighted as TOML (from modeline)
    let out_toml = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")) // Should be highlighted
        .get_output()
        .stdout
        .clone();

    // The two highlighted versions should be different
    assert_ne!(
        out_sh, out_toml,
        "Syntax highlighting from CLI flag and modeline should differ"
    );
}

#[test]
fn cli_pretty_config_file_is_used() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    // Determine config path based on OS and create it. This mimics `dirs::config_dir()`.
    let mut config_dir = home_dir.path().to_path_buf();
    if cfg!(target_os = "macos") {
        config_dir.push("Library/Application Support/rucat");
    } else {
        config_dir.push(".config/rucat");
    }
    std::fs::create_dir_all(&config_dir).unwrap();
    let mut config_file = File::create(config_dir.join("config.toml")).unwrap();
    write!(config_file, "pretty_syntax = 'rust'").unwrap();

    let file = prepare_file(dir.path(), "d.txt", "fn main() {}");

    Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")); // Should be highlighted as Rust due to config
}

#[test]
fn cli_pretty_config_is_overridden_by_flag() {
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    // Set up a config file that specifies 'toml' syntax.
    let mut config_dir = home_dir.path().to_path_buf();
    if cfg!(target_os = "macos") {
        config_dir.push("Library/Application Support/rucat");
    } else {
        config_dir.push(".config/rucat");
    }
    std::fs::create_dir_all(&config_dir).unwrap();
    let mut config_file = File::create(config_dir.join("config.toml")).unwrap();
    write!(config_file, "pretty_syntax = 'toml'").unwrap();

    // A file with shell content.
    let file = prepare_file(dir.path(), "e.txt", "echo 'hello'");

    // Get output when highlighted as shell (from flag). The config file wants TOML.
    let out_sh_from_flag = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty", "--pretty-syntax", "sh"])
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Get output when highlighted as TOML (from config, no flag).
    let out_toml_from_config = Command::cargo_bin("rucat")
        .unwrap()
        .env("HOME", home_dir.path())
        .args(["-f", "pretty"])
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // The two highlighted versions should be different.
    assert_ne!(
        out_sh_from_flag, out_toml_from_config,
        "Highlighting from CLI flag should override config file"
    );
}

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

fn prepare_binary_file(dir: &std::path::Path, name: &str, body: &[u8]) -> std::path::PathBuf {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = File::create(&p).unwrap();
    f.write_all(body).unwrap();
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

#[test]
fn cli_empty_file_is_handled_gracefully() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "empty.txt", "");

    // Markdown should produce a header and an empty code block
    Command::cargo_bin("rucat")
        .unwrap()
        .arg("-f")
        .arg("markdown")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: ")
                .and(predicate::str::contains("empty.txt"))
                // It might be ```txt or ``` depending on extension detection
                .and(predicate::str::contains("```").and(predicate::str::contains("\n```\n"))),
        );

    // Ansi should produce a header and an empty body box
    Command::cargo_bin("rucat")
        .unwrap()
        .arg("-f")
        .arg("ansi")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: ")
                .and(predicate::str::contains("empty.txt"))
                .and(predicate::str::contains('┌'))
                .and(predicate::str::contains('└')),
        );
}

#[test]
fn cli_binary_content_prints_error_and_continues() {
    let dir = tempdir().unwrap();
    let binary_file = prepare_binary_file(dir.path(), "binary.dat", b"\x80\x90\xA0"); // Invalid UTF-8
    let text_file = prepare_file(dir.path(), "text.txt", "hello");

    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&binary_file)
        .arg(&text_file)
        .assert()
        .success() // Should not fail on bad file
        .stdout(
            // The binary file should now produce a hexdump, not an error to stderr
            predicate::str::contains("80 90 a0").and(
                predicate::str::contains("File: ").and(predicate::str::contains("text.txt")), // Check that the second file was processed
            ),
        );
}

#[test]
fn json_from_stdin() {
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "json"])
        .write_stdin("{\"key\": \"value\"}")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""path": "-""#).and(predicate::str::contains(
                r#""content": "{\"key\": \"value\"}""#,
            )),
        );
}

#[test]
fn json_ignores_line_numbers() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");

    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "json", "-n"]) // -n should be ignored
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""content": "hello""#).and(
            predicate::str::contains("1 |").not(), // No line numbers
        ));
}

#[test]
fn strip_more_components_than_exist() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a/b/c.txt", "data");

    Command::cargo_bin("rucat")
        .unwrap()
        .args(["--strip", "99", "-f", "ascii"])
        .arg(&file)
        .assert()
        .success()
        // Should strip down to just the filename
        .stdout(predicate::str::contains("=== c.txt ==="));
}

#[test]
fn strip_has_no_effect_on_stdin() {
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["--strip", "5", "-f", "ascii"])
        .write_stdin("hello")
        .assert()
        .success()
        // The path for stdin is always "-", which should be unaffected by stripping
        .stdout(predicate::str::contains("=== - ==="));
}

#[test]
fn empty_stdin() {
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "markdown"])
        .write_stdin("")
        .assert()
        .success()
        // Should still print a well-formed block for stdin
        .stdout(predicate::str::contains("File: -\n---\n```\n```\n"));
}

#[test]
fn markdown_formatter_ignores_modeline() {
    let dir = tempdir().unwrap();
    // A .txt file with a rust modeline. Markdown should use the extension, not the modeline.
    let file = prepare_file(dir.path(), "a.txt", "fn main() {}\n// vim: ft=rust");

    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "markdown"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("```txt")) // Should be txt from extension
        .stdout(predicate::str::contains("```rust").not()); // Should NOT be rust from modeline
}

#[test]
fn single_dash_is_treated_as_stdin() {
    let dir = tempdir().unwrap();
    let file1 = prepare_file(dir.path(), "a.txt", "file one");
    let file2 = prepare_file(dir.path(), "c.txt", "file two");

    // Test with `-` interspersed with other files
    Command::cargo_bin("rucat")
        .unwrap()
        .args([
            "-f",
            "ascii",
            &file1.to_string_lossy(),
            "-",
            &file2.to_string_lossy(),
        ])
        .write_stdin("stdin content")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a.txt ===\nfile one")
                .and(predicate::str::contains("=== - ===\nstdin content"))
                .and(predicate::str::contains("c.txt ===\nfile two")),
        );
}

#[test]
#[cfg(unix)]
fn non_utf8_filename_is_handled_gracefully() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let dir = tempdir().unwrap();
    // Create a filename with an invalid UTF-8 byte sequence (0xff)
    let invalid_bytes = b"invalid-\xff-name.txt";
    let invalid_os_str = OsStr::from_bytes(invalid_bytes);
    let file_path = dir.path().join(invalid_os_str);

    // We don't need `prepare_file` because we are constructing the path directly
    let mut f = match File::create(&file_path) {
        Ok(file) => file,
        Err(e) => {
            println!(
                "Skipping non-UTF8 filename test: could not create file (likely an OS restriction): {e}"
            );
            return;
        }
    };
    write!(f, "content").unwrap();

    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file_path)
        .assert()
        .success()
        // The invalid byte should be replaced with the Unicode replacement character
        .stdout(predicate::str::contains("invalid-�-name.txt"));
}

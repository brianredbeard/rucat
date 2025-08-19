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
    let mut f = File::create(&p).unwrap();
    write!(f, "{}", body).unwrap();
    p
}

#[test]
fn trailing_numbers_flag() {
    let dir = tempdir().unwrap();
    let file1 = prepare_file(dir.path(), "a.txt", "hello");
    let file2 = prepare_file(dir.path(), "b.txt", "world");
    
    // Test -n after files
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii"])
        .arg(&file1)
        .arg(&file2)
        .arg("-n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 | hello").and(predicate::str::contains("1 | world")));
    
    // Test --numbers after files
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii"])
        .arg(&file1)
        .arg(&file2)
        .arg("--numbers")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 | hello").and(predicate::str::contains("1 | world")));
}

#[test]
fn trailing_format_flag() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test -f after file
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .args(["-f", "xml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<file path="));
    
    // Test --format after file
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .args(["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\":").and(predicate::str::contains("\"content\":")));
}

#[test]
fn mixed_files_and_flags() {
    let dir = tempdir().unwrap();
    let file1 = prepare_file(dir.path(), "a.txt", "first");
    let file2 = prepare_file(dir.path(), "b.txt", "second");
    let file3 = prepare_file(dir.path(), "c.txt", "third");
    
    // Test flags interspersed with files
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file1)
        .arg("-n")
        .arg(&file2)
        .args(["-f", "ascii"])
        .arg(&file3)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 | first")
                .and(predicate::str::contains("1 | second"))
                .and(predicate::str::contains("1 | third"))
                .and(predicate::str::contains("=== ")) // ASCII format header
        );
}

#[test]
fn trailing_strip_flag() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("foo").join("bar");
    std::fs::create_dir_all(&nested).unwrap();
    let file = prepare_file(&nested, "baz.txt", "content");
    
    // Count the actual number of components in the absolute path
    let component_count = file.iter().count();
    // We want to strip all but the last component (the filename)
    let strip_count = if component_count > 1 { component_count - 1 } else { 0 };
    
    // Test --strip after file
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii"])
        .arg(&file)
        .args(["--strip", &strip_count.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== baz.txt ==="));
}

#[test]
fn trailing_width_flags() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "x");
    
    // Test --ansi-width after file
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ansi"])
        .arg(&file)
        .args(["--ansi-width", "30"])
        .assert()
        .success();
    
    // Test --utf8-width after file
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "utf8"])
        .arg(&file)
        .args(["--utf8-width", "40"])
        .assert()
        .success();
}

#[test]
fn trailing_pretty_syntax_flag() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "fn main() {}");
    
    // Test --pretty-syntax after file
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "pretty"])
        .arg(&file)
        .args(["--pretty-syntax", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[")); // Should be highlighted
}

#[test]
fn trailing_unknown_flag_error() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test unknown flag after file
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("--bogus-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown flag '--bogus-flag'"));
}

#[test]
fn trailing_flag_missing_value_error() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test flag requiring value at the end
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("--format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("The '--format' flag requires a value"));
}

#[test]
fn trailing_flag_invalid_value_error() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test invalid format value
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .args(["--format", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid format 'bogus'"));
    
    // Test invalid strip value
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .args(["--strip", "not-a-number"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid value 'not-a-number' for --strip"));
}

#[test]
fn all_flags_after_files() {
    let dir = tempdir().unwrap();
    let file1 = prepare_file(dir.path(), "a.rs", "fn main() {}");
    let file2 = prepare_file(dir.path(), "b.rs", "fn test() {}");
    
    // Test multiple flags all after files
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file1)
        .arg(&file2)
        .args(["-f", "ascii", "-n", "--strip", "0"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 | fn main()")
                .and(predicate::str::contains("1 | fn test()"))
        );
}

#[test]
fn flags_before_and_after_files() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello\nworld");
    
    // Some flags before, some after
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["-f", "ascii"])
        .arg(&file)
        .arg("-n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 | hello")
                .and(predicate::str::contains("2 | world"))
        );
}

#[cfg(feature = "clipboard")]
#[test]
fn trailing_copy_flag() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test -c after file (using test provider to avoid clipboard dependency)
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("-c")
        .args(["--clipboard-provider-for-test", "osc52"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
    
    // Test --copy after file
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("--copy")
        .args(["--clipboard-provider-for-test", "osc52"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[cfg(not(feature = "clipboard"))]
#[test]
fn trailing_copy_flag_without_feature() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    
    // Test that -c after file gives appropriate error when clipboard feature is disabled
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("-c")
        .assert()
        .failure()
        .stderr(predicate::str::contains("The '-c' flag requires the 'clipboard' feature"));
    
    // Test --copy as well
    Command::cargo_bin("rucat")
        .unwrap()
        .arg(&file)
        .arg("--copy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("The '--copy' flag requires the 'clipboard' feature"));
}

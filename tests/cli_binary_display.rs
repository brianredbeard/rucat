// This file is part of rucat. (License details omitted for brevity)
// Copyright (C) 2024 Brian 'redbeard' Harrington

// These tests are Unix-only because they rely on extended attributes (`xattr`).
#![cfg(target_family = "unix")]

use assert_cmd::Command;
use predicates::prelude::*;
use regex::Regex;
use std::path::Path;
use tempfile::tempdir;

const TEXT_ATTR_NAME: &str = "user.comment";
const TEXT_ATTR_VALUE: &str = "this is a text attribute";
const BIN_ATTR_NAME: &str = "user.binary_data";
const BIN_ATTR_VALUE: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];

// Helper to create a test file and set extended attributes on it.
// Returns None if xattrs are not supported on the filesystem.
fn prepare_test_file(dir: &Path) -> Option<std::path::PathBuf> {
    let file_path = dir.join("testfile.txt");
    std::fs::write(&file_path, "file content").unwrap();

    // Setting attributes can fail on some filesystems. If so, we skip the test.
    if xattr::set(&file_path, TEXT_ATTR_NAME, TEXT_ATTR_VALUE.as_bytes()).is_err() {
        println!(
            "Skipping binary display test: extended attributes not supported on this filesystem."
        );
        return None;
    }
    xattr::set(&file_path, BIN_ATTR_NAME, BIN_ATTR_VALUE).unwrap();

    Some(file_path)
}

#[test]
fn default_binary_format_is_hexdump() {
    let dir = tempdir().unwrap();
    if let Some(file) = prepare_test_file(dir.path()) {
        Command::cargo_bin("rucat")
            .unwrap()
            .args(["--stat", "--show-binary-xattrs", "-f", "ascii"])
            .arg(&file)
            .assert()
            .success()
            // Check that the text attribute is displayed normally
            .stdout(predicate::str::contains(TEXT_ATTR_VALUE))
            // Check that the binary attribute is in hex dump format
            .stdout(predicate::str::contains("de ad be ef ca fe ba be"));
    }
}

#[test]
fn binary_format_flag_hexdump() {
    let dir = tempdir().unwrap();
    if let Some(file) = prepare_test_file(dir.path()) {
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "hexdump",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains(TEXT_ATTR_VALUE))
            .stdout(predicate::str::contains("de ad be ef ca fe ba be"));
    }
}

#[test]
fn binary_format_flag_base64() {
    let dir = tempdir().unwrap();
    if let Some(file) = prepare_test_file(dir.path()) {
        let expected_b64 = "3q2+78r+ur4=";
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "base64",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains(TEXT_ATTR_VALUE))
            .stdout(predicate::str::contains(expected_b64));
    }
}

#[test]
fn binary_format_flag_intel_hex() {
    let dir = tempdir().unwrap();
    if let Some(file) = prepare_test_file(dir.path()) {
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "intel-hex",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains(TEXT_ATTR_VALUE))
            // Check for a line of intel hex output
            .stdout(predicate::str::contains(":08000000DEADBEEFCAFEBABE"));
    }
}

#[test]
fn binary_format_flag_raw_hex() {
    let dir = tempdir().unwrap();
    if let Some(file) = prepare_test_file(dir.path()) {
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "raw-hex",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains(TEXT_ATTR_VALUE))
            .stdout(predicate::str::contains("deadbeefcafebabe"));
    }
}

#[test]
fn hex_width_flag_works() {
    let dir = tempdir().unwrap();
    let long_bin_attr: Vec<u8> = (0..32).collect();
    let file_path = dir.path().join("long_xattr.txt");
    std::fs::write(&file_path, "content").unwrap();

    if xattr::set(&file_path, BIN_ATTR_NAME, &long_bin_attr).is_ok() {
        // Default width (16) should produce two lines of hex
        let output_default_raw = Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "hexdump",
            ])
            .arg(&file_path)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_default_str = String::from_utf8(output_default_raw).unwrap();
        println!(
            "--- hex_width_flag_works (default) STDOUT ---\n{output_default_str}\n------------------------------------------"
        );

        let hex_line_re = Regex::new(r"^\s{4}[0-9a-f]{8}:").unwrap();
        let hex_lines_default = output_default_str
            .lines()
            .filter(|line| hex_line_re.is_match(line))
            .count();
        assert_eq!(
            hex_lines_default, 2,
            "Default hex width should produce 2 lines"
        );

        // Width 32 should produce one line of hex
        let output_wide_raw = Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "hexdump",
                "--hex-width",
                "32",
            ])
            .arg(&file_path)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_wide_str = String::from_utf8(output_wide_raw).unwrap();
        println!(
            "--- hex_width_flag_works (wide) STDOUT ---\n{output_wide_str}\n----------------------------------------"
        );

        let hex_lines_wide = output_wide_str
            .lines()
            .filter(|line| hex_line_re.is_match(line))
            .count();
        assert_eq!(hex_lines_wide, 1, "Hex width 32 should produce 1 line");
    }
}

#[test]
fn base64_width_flag_works() {
    let dir = tempdir().unwrap();
    // 80 bytes of data will produce >76 characters of base64, forcing a wrap at the default width
    let long_bin_attr: Vec<u8> = (0..80).collect();
    let file_path = dir.path().join("long_b64.txt");
    std::fs::write(&file_path, "content").unwrap();

    if xattr::set(&file_path, BIN_ATTR_NAME, &long_bin_attr).is_ok() {
        // Default width (76) should produce two lines of base64
        let output_default_raw = Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "base64",
            ])
            .arg(&file_path)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_default_str = String::from_utf8(output_default_raw).unwrap();
        println!(
            "--- base64_width_flag_works (default) STDOUT ---\n{output_default_str}\n--------------------------------------------"
        );

        let b64_line_re = Regex::new(r"^\s{4}[A-Za-z0-9+/=]+$").unwrap();
        let b64_line_count_default = output_default_str
            .lines()
            .filter(|l| b64_line_re.is_match(l))
            .count();
        assert!(
            b64_line_count_default > 1,
            "Default base64 width should wrap to multiple lines. Found {} lines.",
            b64_line_count_default
        );

        // A large width should produce a single line of base64
        let output_wide_raw = Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "ascii",
                "--binary-format",
                "base64",
                "--base64-width",
                "200",
            ])
            .arg(&file_path)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_wide_str = String::from_utf8(output_wide_raw).unwrap();
        println!(
            "--- base64_width_flag_works (wide) STDOUT ---\n{output_wide_str}\n------------------------------------------"
        );

        let b64_line_count_wide = output_wide_str
            .lines()
            .filter(|l| b64_line_re.is_match(l))
            .count();
        assert_eq!(
            b64_line_count_wide, 1,
            "Wide base64 width should not wrap. Found {} lines.",
            b64_line_count_wide
        );
    }
}

#[test]
fn hex_width_zero_is_rejected() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();

    // --hex-width 0 should produce a clap validation error
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["--hex-width", "0"])
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("hex-width"));
}

#[test]
fn base64_width_zero_is_rejected() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();

    // --base64-width 0 should produce a clap validation error
    Command::cargo_bin("rucat")
        .unwrap()
        .args(["--base64-width", "0"])
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("base64-width"));
}

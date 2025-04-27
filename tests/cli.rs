use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn prepare_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();   // ensure nested dirs exist
    }
    let mut f = File::create(&p).unwrap();
    write!(f, "{}", body).unwrap();
    p
}

#[test] fn cli_ascii_numbers() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello\nworld\n");
    Command::cargo_bin("rucat").unwrap()
        .args(["-f", "ascii", "-n"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 | hello")
                              .and(predicate::str::contains("2 | world")));
}

#[test] fn cli_directory_recursion() {
    let dir = tempdir().unwrap();
    prepare_file(dir.path(), "x.rs", "fn main(){}");
    prepare_file(dir.path(), "y.c",  "int main(){}");
    Command::cargo_bin("rucat").unwrap()
        .args(["-f", "markdown"])
        .arg(dir.path())          // pass directory, not files
        .assert()
        .success()
        .stdout(predicate::str::contains("File:")  // header appears twice
                              .count(2));
}

#[test] fn cli_bad_file_reports_error() {
    Command::cargo_bin("rucat").unwrap()
        .args(["no_such_file.txt"])
        .assert()
        .stderr(predicate::str::contains("Error reading"))
        .success();               // program keeps going, exits 0
}

#[test] fn cli_invalid_format_fails() {
    Command::cargo_bin("rucat").unwrap()
        .args(["--format", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}
#[test]
fn cli_strip_components() {
    let dir = tempdir().unwrap();
    let p = prepare_file(dir.path(), "foo/bar/baz.h", "x"); // create sub-dirs
    Command::cargo_bin("rucat").unwrap()
        .args(["-f", "ascii", "--strip", "2"])
        .arg(&p)
        .assert()
        .success()
        // after --strip 2 the printed header must end with “baz.h”
        .stdout(predicate::str::contains("baz.h"));
}

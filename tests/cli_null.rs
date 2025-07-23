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
use predicates::str::contains;
use tempfile::tempdir;

fn prepare(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, name).unwrap();
    p
}

#[test]
fn null_list_is_respected() {
    let dir = tempdir().unwrap();
    let f1 = prepare(dir.path(), "a.txt");
    let f2 = prepare(dir.path(), "b.txt");

    let input = format!("{}\0{}", f1.display(), f2.display());

    Command::cargo_bin("rucat").unwrap()
        .args(["-0", "-f", "ascii"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("=== ").count(2)); // both headers printed
}

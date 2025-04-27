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

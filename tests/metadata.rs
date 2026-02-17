use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

pub mod metadata_fixtures;

// Test helper to create a file with specific content.
fn prepare_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = File::create(&p).unwrap();
    write!(f, "{body}").unwrap();
    p
}

#[test]
fn stat_flag_prints_metadata_header() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "a.txt", "hello");
    Command::cargo_bin("rucat")
        .unwrap()
        .arg("--stat")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: ")
                .and(predicate::str::contains("a.txt"))
                .and(predicate::str::contains("Size: 5 bytes"))
                .and(predicate::str::contains("Permissions:")),
        );
}

#[test]
#[cfg(target_family = "unix")]
fn stat_flag_prints_selinux_context_if_present() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "b.txt", "selinux test");

    let attr_name = "security.selinux";
    let attr_value = "unconfined_u:object_r:user_home_t:s0";

    // Set the extended attribute. This might fail if the filesystem doesn't support it,
    // so we'll conditionally run the check.
    if xattr::set(&file, attr_name, attr_value.as_bytes()).is_ok() {
        Command::cargo_bin("rucat")
            .unwrap()
            .arg("--stat")
            .arg(&file)
            .assert()
            .success()
            .stdout(
                predicate::str::contains("SELinux Context:")
                    .and(predicate::str::contains(attr_value)),
            );
    } else {
        println!("Skipping SELinux test: extended attributes not supported on this filesystem.");
    }
}

#[test]
#[cfg(target_family = "unix")]
fn stat_flag_prints_posix_acls_if_present() {
    use exacl::{AclEntry, Perm, setfacl};
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "c.txt", "acl test");
    // Attempt to get the base ACL. This will fail if not supported.
    let mut acl = match exacl::getfacl(&file, None) {
        Ok(acl) => acl,
        Err(e) => {
            println!("Skipping POSIX ACL test: not supported on this filesystem ({e}).");
            return;
        }
    };
    // Add a new entry. We use the 'staff' group as it's common on macOS and many Linux systems.
    acl.push(AclEntry::allow_group("staff", Perm::READ, None));
    // Set the new ACL.
    if let Err(e) = setfacl(&[&file], &acl, None) {
        println!("Skipping POSIX ACL test: failed to set ACL ({e}).");
        return;
    }
    Command::cargo_bin("rucat")
        .unwrap()
        .arg("--stat")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("POSIX ACLs:")
                .and(predicate::str::contains("allow::group:staff:read")),
        );
}

#[test]
fn stat_flag_works_with_json_output() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "c.json", "{\"key\":\"value\"}");
    Command::cargo_bin("rucat")
        .unwrap()
        .arg("--stat")
        .arg("-f")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"path\":")
                .and(predicate::str::contains("\"content\":"))
                .and(predicate::str::contains("\"metadata\":"))
                .and(predicate::str::contains("\"size\":"))
                .and(predicate::str::contains("\"permissions\":"))
                .and(predicate::str::contains("\"created\":")),
        );
}

#[test]
fn json_output_no_metadata() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "c.json", "{\"key\":\"value\"}");
    Command::cargo_bin("rucat")
        .unwrap()
        // NO --stat flag
        .arg("-f")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"path\":").and(
                predicate::str::contains("\"content\":")
                    .and(predicate::str::contains("\"metadata\":").not()),
            ),
        );
}

#[test]
#[cfg(target_family = "unix")]
fn show_binary_xattrs_flag_works() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "d.txt", "binary xattr test");

    let attr_name = "user.binary_data";
    let attr_value: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

    if xattr::set(&file, attr_name, &attr_value).is_ok() {
        // Test without the flag: binary data should be hidden
        Command::cargo_bin("rucat")
            .unwrap()
            .arg("--stat")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("user.binary_data").not());

        // Test with the flag: binary data should be shown in default hexdump format
        Command::cargo_bin("rucat")
            .unwrap()
            .args(["--stat", "--show-binary-xattrs"])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("de ad be ef"));
    } else {
        println!(
            "Skipping binary xattr test: extended attributes not supported on this filesystem."
        );
    }
}

#[test]
#[cfg(target_os = "macos")]
fn macos_fixture_is_formatted() {
    use rucat::binary_display::{BinaryFormatOptions, BinaryOutputFormat};
    use rucat::formatters::{Formatter, markdown::Markdown};
    use std::path::Path;
    let fixture = metadata_fixtures::macos_fixture();
    let mut fmt = Markdown {
        line_numbers: false,
    };
    let mut buf = Vec::new();
    let options = BinaryFormatOptions {
        show_binary: true,
        format: BinaryOutputFormat::default(),
        hex_width: 16,
        base64_width: 76,
    };
    fmt.write(
        Path::new(&fixture.path),
        "content",
        Some(&fixture),
        &options,
        &mut buf,
    )
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("com.apple.metadata:kMDItemFinderComment: test comment"));
    assert!(out.contains("Size: 12 bytes"));
    assert!(out.contains("Permissions: -rw-r--r--"));
    assert!(out.contains("Created: Jan  1 10:00:00 2024"));
}

#[test]
#[cfg(target_family = "unix")]
fn show_binary_xattrs_base64_in_json_xml() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "e.txt", "base64 test");

    let attr_name = "user.binary_data";
    let attr_value: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let expected_b64 = "3q2+7w==";

    if xattr::set(&file, attr_name, &attr_value).is_ok() {
        // Test JSON output
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "json",
                "--binary-format",
                "base64",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(
                predicate::str::contains(attr_name).and(predicate::str::contains(expected_b64)),
            );

        // Test XML output
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--stat",
                "--show-binary-xattrs",
                "-f",
                "xml",
                "--binary-format",
                "base64",
            ])
            .arg(&file)
            .assert()
            .success()
            .stdout(
                predicate::str::contains(attr_name).and(predicate::str::contains(expected_b64)),
            );
    } else {
        println!(
            "Skipping base64 xattr test: extended attributes not supported on this filesystem."
        );
    }
}

#[test]
#[cfg(windows)]
fn stat_flag_windows_readonly_permission() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "readonly.txt", "data");

    let mut perms = std::fs::metadata(&file).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&file, perms).unwrap();

    Command::cargo_bin("rucat")
        .unwrap()
        .arg("--stat")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Permissions: readonly"));
}

#[test]
#[cfg(target_family = "unix")]
fn large_binary_xattr_is_truncated() {
    let dir = tempdir().unwrap();
    let file = prepare_file(dir.path(), "large_xattr.txt", "content");

    let attr_name = "user.large_binary_data";
    // Create data larger than the 1MB limit in `metadata.rs`
    let large_value: Vec<u8> = vec![0xAB; 1_048_576 + 100];

    if xattr::set(&file, attr_name, &large_value).is_ok() {
        Command::cargo_bin("rucat")
            .unwrap()
            .args(["--stat", "--show-binary-xattrs", "-f", "ascii"])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("... (truncated)"));
    } else {
        println!(
            "Skipping large xattr test: extended attributes not supported on this filesystem."
        );
    }
}

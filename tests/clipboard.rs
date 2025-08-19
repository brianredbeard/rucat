#[cfg(feature = "clipboard")]
mod clipboard_tests {
    // This file is part of rucat. (License details omitted for brevity)
    // Copyright (C) 2024 Brian 'redbeard' Harrington
    use assert_cmd::Command;
    use base64::{Engine as _, engine::general_purpose};
    use predicates::prelude::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Test helper to create a file with specific content.
    fn prepare_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        let mut f = File::create(&p).unwrap();
        write!(f, "{body}").unwrap();
        p
    }

    // This test is cross-platform. It uses the hidden test flag to force an OSC provider,
    // ensuring the --copy flag doesn't crash on CI runners for any OS.
    #[test]
    fn copy_flag_still_prints_to_stdout() {
        let dir = tempdir().unwrap();
        let file = prepare_file(dir.path(), "a.txt", "hello world");
        Command::cargo_bin("rucat")
            .unwrap()
            .arg("--copy")
            .args(["--clipboard-provider-for-test", "osc52"])
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("hello world"));
    }

    // This test is cross-platform. It forces the OSC 5522 provider via the hidden flag
    // to verify the correct escape sequence is generated on any OS.
    #[test]
    fn force_osc5522_provider_works_on_all_platforms() {
        let input_content = "hello kitty";
        // rucat defaults to Markdown format. When reading from stdin, the path is "-".
        let expected_formatted_output = format!("---\nFile: -\n---\n```\n{input_content}\n```\n");
        let b64_content = general_purpose::STANDARD.encode(&expected_formatted_output);

        Command::cargo_bin("rucat")
            .unwrap()
            .write_stdin(input_content)
            .args(["--copy", "--clipboard-provider-for-test", "osc5522"])
            .assert()
            .success()
            .stdout(
                // The app prints the formatted output to stdout...
                predicate::str::contains(&expected_formatted_output)
                    // ...and also prints an OSC sequence containing the base64 of that same output.
                    .and(predicate::str::contains(&b64_content)),
            );
    }

    // This test is cross-platform. It forces the OSC 52 provider via the hidden flag
    // to verify the correct escape sequence is generated on any OS.
    #[test]
    fn force_osc52_provider_works_on_all_platforms() {
        let input_content = "hello tmux";
        let expected_formatted_output = format!("---\nFile: -\n---\n```\n{input_content}\n```\n");
        let b64_content = general_purpose::STANDARD.encode(&expected_formatted_output);

        Command::cargo_bin("rucat")
            .unwrap()
            .write_stdin(input_content)
            .args(["--copy", "--clipboard-provider-for-test", "osc52"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains(&expected_formatted_output)
                    .and(predicate::str::contains(&b64_content)),
            );
    }

    // This test is cross-platform. It verifies that the hidden test flag logic
    // itself fails gracefully when given a bad provider name.
    #[test]
    fn invalid_test_provider_fails_gracefully() {
        let dir = tempdir().unwrap();
        let file = prepare_file(dir.path(), "a.txt", "invalid provider");
        Command::cargo_bin("rucat")
            .unwrap()
            .args([
                "--copy",
                "--clipboard-provider-for-test",
                "bogus-provider-name",
            ])
            .arg(&file)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Invalid test provider"));
    }

    // This test is platform-specific. It tests the auto-detection failure case, which
    // can only be reliably triggered in a headless Linux environment.
    #[test]
    #[cfg(target_os = "linux")]
    fn auto_detection_fails_gracefully_on_headless_linux() {
        let dir = tempdir().unwrap();
        let file = prepare_file(dir.path(), "a.txt", "no clipboard");
        Command::cargo_bin("rucat")
            .unwrap()
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("TERM")
            .arg("--copy")
            .arg(&file)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Failed to initialize clipboard"));
    }
}

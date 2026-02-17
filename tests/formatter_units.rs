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
use rucat::binary_display::{BinaryFormatOptions, BinaryOutputFormat};
use rucat::formatters::{
    Formatter, ansi::Ansi, ascii::Ascii, markdown::Markdown, pretty::Pretty, utf8::Utf8, xml::Xml,
};
use rucat::metadata::{FileMetadata, PlatformMetadata, UnixMetadata};
use std::collections::HashMap;
use std::path::Path;

fn capture_with_path<F: Formatter>(
    fmt: &mut F,
    path: &Path,
    content: &str,
    metadata: Option<&FileMetadata>,
) -> String {
    let mut buf = Vec::new();
    let options = BinaryFormatOptions {
        show_binary: false,
        format: BinaryOutputFormat::default(),
        hex_width: 16,
        base64_width: 76,
    };
    fmt.write(path, content, metadata, &options, &mut buf)
        .unwrap();
    String::from_utf8(buf).unwrap()
}

// Convenience wrapper for old tests that don't care about the path.
fn capture<F: Formatter>(fmt: &mut F, content: &str) -> String {
    capture_with_path(fmt, Path::new("foo.rs"), content, None)
}

#[test]
fn ascii_numbers() {
    let mut fmt = Ascii { line_numbers: true };
    let out = capture(&mut fmt, "abc\n");
    assert!(out.contains("1 | abc"));
}

#[test]
fn ascii_plain() {
    let mut fmt = Ascii {
        line_numbers: false,
    };
    let out = capture(&mut fmt, "abc");
    assert!(!out.contains('|'));
}

#[test]
fn ansi_fixed_width() {
    let mut fmt = Ansi {
        width: 20,
        line_numbers: false,
    };
    let out = capture(&mut fmt, "abc");
    let first = out.lines().next().unwrap();
    let last = out.lines().last().unwrap();
    assert_eq!(first.len(), last.len()); // borders equal
}

#[test]
fn utf8_numbers_width() {
    let mut fmt = Utf8 {
        width: 30,
        line_numbers: true,
    };
    let out = capture(&mut fmt, "x");
    assert!(out.contains("1 │ x"));
}

#[test]
fn markdown_block() {
    let mut fmt = Markdown { line_numbers: true };
    let out = capture(&mut fmt, "fn main(){}");
    assert!(out.starts_with("---\nFile:"));
    assert!(out.contains("```rs") || out.contains("```")); // ext may be blank
}

#[test]
fn xml_numbers_vs_plain() {
    let mut fmt_with = Xml { line_numbers: true };
    let with = capture(&mut fmt_with, "a\nb");
    let mut fmt_no = Xml {
        line_numbers: false,
    };
    let no = capture(&mut fmt_no, "a\nb");
    assert!(with.contains("<line no=\"1\">"));
    assert!(!no.contains("<line no=\"1\">"));
}

#[test]
fn xml_escaping() {
    let mut fmt = Xml {
        line_numbers: false,
    };
    let out = capture(&mut fmt, "<tag>&\"'</tag>");
    assert!(out.contains("&lt;tag&gt;&amp;&quot;&apos;&lt;/tag&gt;"));
}

#[test]
fn xml_path_escaping() {
    let mut fmt = Xml {
        line_numbers: false,
    };
    // Test filename with XML special characters: ", <, >, &, '
    let path = Path::new("file<name>&\"test'.txt");
    let out = capture_with_path(&mut fmt, path, "content", None);

    // Path attribute should have escaped characters
    assert!(out.contains("&lt;")); // < escaped to &lt;
    assert!(out.contains("&gt;")); // > escaped to &gt;
    assert!(out.contains("&amp;")); // & escaped to &amp;
    assert!(out.contains("&quot;")); // " escaped to &quot;
    assert!(out.contains("&apos;")); // ' escaped to &apos;

    // Should NOT contain unescaped characters in path attribute
    assert!(!out.contains("path=\"file<")); // raw < should not appear in attribute
}

#[test]
fn xml_comment_sanitization() {
    let mut fmt = Xml {
        line_numbers: false,
    };
    // Create metadata with --> and --- sequences
    let mut meta = mock_metadata();
    meta.path = "file-->test---name.txt".to_string();
    let path = Path::new("test.txt");
    let out = capture_with_path(&mut fmt, path, "content", Some(&meta));

    // The metadata path contains --> which should be sanitized to - ->
    // and --- which should be sanitized to - - -
    assert!(out.contains("- ->")); // --> becomes - ->
    assert!(out.contains("- - -")); // --- becomes - - -
    // Exactly one --> should exist (the legitimate closing tag, not from data)
    assert_eq!(
        out.matches("-->").count(),
        1,
        "metadata content should not produce additional --> sequences"
    );
}

#[test]
fn pretty_highlighting() {
    let mut fmt = Pretty {
        line_numbers: true,
        syntax_override: None,
    };
    let out = capture(&mut fmt, "fn main() {}");
    assert!(out.contains("1 │")); // line number
    assert!(out.contains("\x1b[")); // ansi escape code
}

#[test]
fn pretty_syntax_override() {
    // The content "key = 'value'" should be highlighted as TOML, despite the .rs extension.
    let mut fmt = Pretty {
        line_numbers: false,
        syntax_override: Some("toml".to_string()),
    };
    let out = capture_with_path(&mut fmt, Path::new("foo.rs"), "key = 'value'", None);

    // For comparison, highlight as plain text (by giving an unknown extension and no override).
    let mut fmt_plain = Pretty {
        line_numbers: false,
        syntax_override: None,
    };
    let out_plain = capture_with_path(&mut fmt_plain, Path::new("foo.txt"), "key = 'value'", None);

    assert!(out.contains("\x1b[")); // Should be highlighted.
    assert_ne!(out, out_plain); // And should be different from plain text.
    assert_ne!(out, "key = 'value'\n");
}

#[test]
fn pretty_modeline_detection() {
    let content = "fn main() {}\n// vim: ft=rust";
    let mut fmt = Pretty {
        line_numbers: false,
        syntax_override: None,
    };

    // Use a .txt extension to prove modeline is being used over the file extension.
    let out = capture_with_path(&mut fmt, Path::new("foo.txt"), content, None);

    // For comparison, format the same content without the modeline.
    let out_plain = capture_with_path(&mut fmt, Path::new("foo.txt"), "fn main() {}", None);

    assert!(out.contains("\x1b[")); // Should be highlighted as rust.
    assert_ne!(out, out_plain); // Should be different from plain text version.
}

fn mock_metadata() -> FileMetadata {
    FileMetadata {
        path: "test.txt".to_string(),
        size: 42,
        permissions: "-rwxr-xr-x".to_string(),
        created: Some("Jan  1 00:00:00 1970".to_string()),
        modified: Some("Jan  1 00:00:00 1970".to_string()),
        accessed: Some("Jan  1 00:00:00 1970".to_string()),
        extended_attributes: {
            let mut map = HashMap::new();
            map.insert("user.comment".to_string(), b"test comment".to_vec());
            map
        },
        security_context: Some("unconfined_u:object_r:user_home_t:s0".to_string()),
        platform_specific: PlatformMetadata::Unix(UnixMetadata {
            selinux_context: Some("unconfined_u:object_r:user_home_t:s0".to_string()),
            posix_acls: None,
        }),
    }
}

macro_rules! test_formatter_stat_behavior {
    ($name:ident, $formatter:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;
            use predicates::prelude::*;

            #[test]
            fn content_and_metadata_are_shown() {
                let mut fmt = $formatter;
                let meta = mock_metadata();
                let content = "file content";
                let output =
                    capture_with_path(&mut fmt, Path::new("test.txt"), content, Some(&meta));

                let predicate = predicate::str::contains("SELinux Context")
                    .and(predicate::str::contains(content));
                assert!(predicate.eval(&output));
            }

            #[test]
            fn only_content_is_shown() {
                let mut fmt = $formatter;
                let content = "file content";
                let output = capture_with_path(&mut fmt, Path::new("test.txt"), content, None);

                let predicate = predicate::str::contains("SELinux Context")
                    .not()
                    .and(predicate::str::contains(content));
                assert!(predicate.eval(&output));
            }
        }
    };
}

test_formatter_stat_behavior!(
    ansi_stat_behavior,
    Ansi {
        width: 80,
        line_numbers: false
    }
);
test_formatter_stat_behavior!(
    ascii_stat_behavior,
    Ascii {
        line_numbers: false
    }
);
test_formatter_stat_behavior!(
    markdown_stat_behavior,
    Markdown {
        line_numbers: false
    }
);
test_formatter_stat_behavior!(
    pretty_stat_behavior,
    Pretty {
        line_numbers: false,
        syntax_override: None
    }
);
test_formatter_stat_behavior!(
    utf8_stat_behavior,
    Utf8 {
        width: 80,
        line_numbers: false
    }
);
test_formatter_stat_behavior!(
    xml_stat_behavior,
    Xml {
        line_numbers: false
    }
);

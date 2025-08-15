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
use rucat::formatters::{
    Formatter, ansi::Ansi, ascii::Ascii, markdown::Markdown, pretty::Pretty, utf8::Utf8, xml::Xml,
};
use std::path::Path;

fn capture_with_path<F: Formatter>(fmt: &F, path: &Path, content: &str) -> String {
    let mut buf = Vec::new();
    fmt.write(path, content, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

// Convenience wrapper for old tests that don't care about the path.
fn capture<F: Formatter>(fmt: &F, content: &str) -> String {
    capture_with_path(fmt, Path::new("foo.rs"), content)
}

#[test]
fn ascii_numbers() {
    let out = capture(&Ascii { line_numbers: true }, "abc\n");
    assert!(out.contains("1 | abc"));
}

#[test]
fn ascii_plain() {
    let out = capture(
        &Ascii {
            line_numbers: false,
        },
        "abc",
    );
    assert!(!out.contains('|'));
}

#[test]
fn ansi_fixed_width() {
    let out = capture(
        &Ansi {
            width: 20,
            line_numbers: false,
        },
        "abc",
    );
    let first = out.lines().next().unwrap();
    let last = out.lines().last().unwrap();
    assert_eq!(first.len(), last.len()); // borders equal
}

#[test]
fn utf8_numbers_width() {
    let out = capture(
        &Utf8 {
            width: 30,
            line_numbers: true,
        },
        "x",
    );
    assert!(out.contains("1 │ x"));
}

#[test]
fn markdown_block() {
    let out = capture(&Markdown { line_numbers: true }, "fn main(){}");
    assert!(out.starts_with("---\nFile:"));
    assert!(out.contains("```rs") || out.contains("```")); // ext may be blank
}

#[test]
fn xml_numbers_vs_plain() {
    let with = capture(&Xml { line_numbers: true }, "a\nb");
    let no = capture(
        &Xml {
            line_numbers: false,
        },
        "a\nb",
    );
    assert!(with.contains("<line no=\"1\">"));
    assert!(!no.contains("<line no=\"1\">"));
}

#[test]
fn pretty_highlighting() {
    let out = capture(
        &Pretty {
            line_numbers: true,
            syntax_override: None,
        },
        "fn main() {}",
    );
    assert!(out.contains("1 │")); // line number
    assert!(out.contains("\x1b[")); // ansi escape code
}

#[test]
fn pretty_syntax_override() {
    // The content "key = 'value'" should be highlighted as TOML, despite the .rs extension.
    let fmt = Pretty {
        line_numbers: false,
        syntax_override: Some("toml".to_string()),
    };
    let out = capture_with_path(&fmt, Path::new("foo.rs"), "key = 'value'");

    // For comparison, highlight as plain text (by giving an unknown extension and no override).
    let fmt_plain = Pretty {
        line_numbers: false,
        syntax_override: None,
    };
    let out_plain = capture_with_path(&fmt_plain, Path::new("foo.txt"), "key = 'value'");

    assert!(out.contains("\x1b[")); // Should be highlighted.
    assert_ne!(out, out_plain); // And should be different from plain text.
    assert_ne!(out, "key = 'value'\n");
}

#[test]
fn pretty_modeline_detection() {
    let content = "fn main() {}\n// vim: ft=rust";
    let fmt = Pretty {
        line_numbers: false,
        syntax_override: None,
    };

    // Use a .txt extension to prove modeline is being used over the file extension.
    let out = capture_with_path(&fmt, Path::new("foo.txt"), content);

    // For comparison, format the same content without the modeline.
    let out_plain = capture_with_path(&fmt, Path::new("foo.txt"), "fn main() {}");

    assert!(out.contains("\x1b[")); // Should be highlighted as rust.
    assert_ne!(out, out_plain); // Should be different from plain text version.
}

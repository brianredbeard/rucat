use rucat::formatters::{
    ascii::Ascii, ansi::Ansi, utf8::Utf8,
    markdown::Markdown, xml::Xml, Formatter,
};
use std::path::Path;

fn capture<F: Formatter>(fmt: &F, content: &str) -> String {
    let mut buf = Vec::new();
    fmt.write(Path::new("foo.rs"), content, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test] fn ascii_numbers() {
    let out = capture(&Ascii { line_numbers: true }, "abc\n");
    assert!(out.contains("1 | abc"));
}

#[test] fn ascii_plain() {
    let out = capture(&Ascii { line_numbers: false }, "abc");
    assert!(!out.contains('|'));
}

#[test] fn ansi_fixed_width() {
    let out = capture(&Ansi { width: 20, line_numbers: false }, "abc");
    let first = out.lines().next().unwrap();
    let last  = out.lines().last().unwrap();
    assert_eq!(first.len(), last.len());          // borders equal
}

#[test] fn utf8_numbers_width() {
    let out = capture(&Utf8 { width: 30, line_numbers: true }, "x");
    assert!(out.contains("1 │ x"));
}

#[test] fn markdown_block() {
    let out = capture(&Markdown { line_numbers: true }, "fn main(){}");
    assert!(out.starts_with("---\nFile:"));
    assert!(out.contains("```rs") || out.contains("```")); // ext may be blank
}

#[test] fn xml_numbers_vs_plain() {
    let with = capture(&Xml { line_numbers: true },  "a\nb");
    let no   = capture(&Xml { line_numbers: false }, "a\nb");
    assert!(with.contains("<line no=\"1\">"));
    assert!(!no.contains("<line no=\"1\">"));
}

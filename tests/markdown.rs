use rucat::formatters::{markdown::Markdown, Formatter};
use std::path::Path;

#[test]
fn md_basic() {
    let mut buf = Vec::new();
    let fmt = Markdown { line_numbers: false };   // instantiate struct
    fmt.write(Path::new("foo.rs"), "fn main(){}", &mut buf).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("```rs"));
}

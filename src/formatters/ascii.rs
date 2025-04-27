use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

/// Very simple output: `=== path ===` header + raw body.
pub struct Ascii;

impl Formatter for Ascii {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "=== {} ===", path.display())?;
        write!(w, "{}", content)?;
        if !content.ends_with('\n') {
            writeln!(w)?;
        }
        Ok(())
    }
}

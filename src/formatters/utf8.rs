use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Utf8;

impl Formatter for Utf8 {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "=== {} ===", path.display())?;
        write!(w, "{}", content)?;
        if !content.ends_with('\n') {
            writeln!(w)?;
        }
        Ok(())
    }
}

use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

/// Very simple output: `=== path ===` header + raw body.
pub struct Ascii { pub line_numbers: bool }

impl Formatter for Ascii {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "=== {} ===", path.display())?;
        let total = content.lines().count();
        let width = if self.line_numbers { total.to_string().len() } else { 0 };
        for (idx, line) in content.lines().enumerate() {
            if self.line_numbers {
                //  number | content   (ASCII separator)
                writeln!(w, "{:>w$} | {}", idx + 1, line, w = width)?;
            } else {
                writeln!(w, "{}", line)?;
            }
        }
        Ok(())
    }
}

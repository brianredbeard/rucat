use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

/// Fancy UTF-8 box-drawing formatter (configurable width).
pub struct Utf8 {
    pub width: usize,
    pub line_numbers: bool,
}

impl Formatter for Utf8 {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let width = self.width;
        let hr = "─".repeat(width);

        // Header line (truncate/pad as needed)
        let mut hdr = format!(" File: {} ", path.display());
        hdr.truncate(width);
        if hdr.len() < width { hdr.push_str(&" ".repeat(width - hdr.len())); }

        writeln!(w, "┌{}┐", hr)?;        // top border
        writeln!(w, "│{}│", hdr)?;       // header
        writeln!(w, "├{}┤", hr)?;        // separator

        let total = content.lines().count();
        let digits = if self.line_numbers { total.to_string().len() } else { 0 };

        for (idx, raw) in content.lines().enumerate() {
            //  number │ content   (UTF-8 vertical bar separator inside frame)
            let mut l = if self.line_numbers {
                format!("{:>w$} │ {}", idx + 1, raw, w = digits)
            } else {
                raw.to_owned()
            };
            l.truncate(width);
            if l.len() < width { l.push_str(&" ".repeat(width - l.len())); }
            writeln!(w, "│{}│", l)?;
        }
        writeln!(w, "└{}┘", hr)?;        // bottom border
        Ok(())
    }
}

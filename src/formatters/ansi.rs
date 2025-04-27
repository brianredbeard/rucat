use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Ansi { pub width: usize }

impl Formatter for Ansi {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let width = self.width;
        let horizontal_line = "─".repeat(width);
        let file_info = format!(" File: {}", path.display());
        let file_info_padding_len = width.saturating_sub(file_info.len() + 1);
        let file_info_padded = format!(" {}{}", file_info, " ".repeat(file_info_padding_len));

        writeln!(w, "┬{}─", horizontal_line)?;
        writeln!(w, "│{}│", file_info_padded)?;
        writeln!(w, "┼{}─", horizontal_line)?;

        for line in content.lines() {
            let line_padding_len = width.saturating_sub(line.len() + 1);
            let line_padded = format!(" {}{}", line, " ".repeat(line_padding_len));
            writeln!(w, "│{}│", line_padded)?;
        }

        writeln!(w, "┴{}─", horizontal_line)?;
        Ok(())
    }
}

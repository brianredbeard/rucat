use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Ansi { pub width: usize, pub line_numbers: bool }

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

        let total = content.lines().count();
        let digits = if self.line_numbers { total.to_string().len() } else { 0 };
        for (idx, line) in content.lines().enumerate() {
            //  number │ content   (graphical separator for ANSI box)
            let base = if self.line_numbers {
                format!("{:>w$} │ {}", idx + 1, line, w = digits)
            } else {
                line.to_owned()
            };
            let line_padding_len = width.saturating_sub(base.len() + 1);
            let line_padded = format!(" {}{}", base, " ".repeat(line_padding_len));
            writeln!(w, "│{}│", line_padded)?;
        }

        writeln!(w, "┴{}─", horizontal_line)?;
        Ok(())
    }
}

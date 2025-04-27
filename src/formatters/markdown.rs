use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Markdown { pub line_numbers: bool }

impl Formatter for Markdown {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let extension = path.extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
        writeln!(w, "---\nFile: {}\n---", path.display())?;
        writeln!(w, "```{}", extension)?;
        let total = content.lines().count();
        let digits = if self.line_numbers { total.to_string().len() } else { 0 };
        for (idx, line) in content.lines().enumerate() {
            if self.line_numbers {
                writeln!(w, "{:>w$} {}", idx + 1, line, w = digits)?;
            } else {
                writeln!(w, "{}", line)?;
            }
        }
        writeln!(w, "```")?;
        Ok(())
    }
}

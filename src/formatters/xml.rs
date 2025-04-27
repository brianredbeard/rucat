use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Xml { pub line_numbers: bool }

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

impl Formatter for Xml {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        if self.line_numbers {
            writeln!(w, "<file path=\"{}\">", path.display())?;
            for (idx, line) in content.lines().enumerate() {
                writeln!(w,
                    "  <line no=\"{}\">{}</line>",
                    idx + 1,
                    esc(line)
                )?;
            }
            writeln!(w, "</file>")?;
        } else {
            writeln!(w,
                "<file path=\"{}\">{}</file>",
                path.display(),
                esc(content)
            )?;
        }
        Ok(())
    }
}

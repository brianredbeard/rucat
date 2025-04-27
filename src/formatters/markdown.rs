use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Markdown;

impl Formatter for Markdown {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let extension = path.extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
        writeln!(w, "---\nFile: {}\n---", path.display())?;
        writeln!(w, "```{}\n{}\n```", extension, content)?;
        Ok(())
    }
}

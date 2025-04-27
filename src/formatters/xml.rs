use super::Formatter;
use std::io::{self, Write};
use std::path::Path;

pub struct Xml;

impl Formatter for Xml {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let escaped_content = content.replace("&", "&amp;")
                                     .replace("<", "&lt;")
                                     .replace(">", "&gt;")
                                     .replace("\"", "&quot;")
                                     .replace("'", "&apos;");
        writeln!(w, "<file path=\"{}\">{}</file>", path.display(), escaped_content)?;
        Ok(())
    }
}

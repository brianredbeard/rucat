use super::Formatter;
use crate::json_entry::FileEntry;
use crate::metadata::FileMetadata;
use std::io::{self, Write};
use std::path::Path;

pub struct Json {
    first: bool,
}

impl Json {
    pub fn new() -> Self {
        Self { first: true }
    }
}

impl Default for Json {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for Json {
    fn start(&mut self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "[")
    }

    fn write(
        &mut self,
        path: &Path,
        content: &str,
        metadata: Option<&FileMetadata>,
        options: &crate::binary_display::BinaryFormatOptions,
        w: &mut dyn Write,
    ) -> io::Result<()> {
        if !self.first {
            // Add a comma and newline before each subsequent entry
            writeln!(w, ",")?;
        }
        self.first = false;

        let entry = FileEntry::new(
            path,
            content.to_string(),
            metadata.cloned(), // Cloned because FileEntry takes ownership
            options.show_binary,
        );

        // Use a temp buffer to pretty-print, then indent each line to fit into the array structure.
        let pretty_json = serde_json::to_string_pretty(&entry).map_err(io::Error::other)?;
        for line in pretty_json.lines() {
            // Indent each line with two spaces
            writeln!(w, "  {line}")?;
        }
        Ok(())
    }

    fn finish(&mut self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "\n]")
    }
}

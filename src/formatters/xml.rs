// This file is part of rucat.
//
// rucat is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rucat is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rucat.  If not, see <https://www.gnu.org/licenses/>.
//
// Copyright (C) 2024 Brian 'redbeard' Harrington
use super::Formatter;
use crate::metadata::FileMetadata;
use std::io::{self, Write};
use std::path::Path;

pub struct Xml {
    pub line_numbers: bool,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn sanitize_xml_comment(s: &str) -> String {
    // XML spec prohibits -- in comments. Replace with - - (space inserted).
    // This also prevents --> breakout since it contains --.
    // For multiple consecutive hyphens like ---, apply repeatedly until stable.
    let mut result = s.to_string();
    while result.contains("--") {
        result = result.replace("--", "- -");
    }
    result
}

impl Formatter for Xml {
    fn write(
        &mut self,
        path: &Path,
        content: &str,
        metadata: Option<&FileMetadata>,
        options: &crate::binary_display::BinaryFormatOptions,
        w: &mut dyn Write,
    ) -> io::Result<()> {
        if let Some(meta) = metadata {
            let formatted_meta = meta.format_for_display(options);
            let sanitized_meta = sanitize_xml_comment(&formatted_meta);
            writeln!(w, "<!--\n{sanitized_meta}-->")?;
        }

        if self.line_numbers {
            writeln!(w, "<file path=\"{}\">", esc(&path.display().to_string()))?;
            for (idx, line) in content.lines().enumerate() {
                writeln!(w, "  <line no=\"{}\">{}</line>", idx + 1, esc(line))?;
            }
            writeln!(w, "</file>")?;
        } else {
            writeln!(
                w,
                "<file path=\"{}\">{}</file>",
                esc(&path.display().to_string()),
                esc(content)
            )?;
        }
        Ok(())
    }
}

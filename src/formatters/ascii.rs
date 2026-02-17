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

/// Very simple output: `=== path ===` header + raw body.
pub struct Ascii {
    pub line_numbers: bool,
}

impl Formatter for Ascii {
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
            writeln!(w, "{formatted_meta}")?;
        } else {
            writeln!(w, "=== {} ===", path.display())?;
        }
        let total = content.lines().count();
        let width = if self.line_numbers {
            total.to_string().len()
        } else {
            0
        };
        for (idx, line) in content.lines().enumerate() {
            if self.line_numbers {
                //  number | content   (ASCII separator)
                writeln!(w, "{:>w$} | {}", idx + 1, line, w = width)?;
            } else {
                writeln!(w, "{line}")?;
            }
        }
        Ok(())
    }
}

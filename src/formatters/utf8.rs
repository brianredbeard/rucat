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
use std::io::{self, Write};
use std::path::Path;

/// Fancy UTF-8 box-drawing formatter (configurable width).
pub struct Utf8 {
    pub width: usize,
    pub line_numbers: bool,
}

impl Formatter for Utf8 {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let digits = if self.line_numbers {
            content.lines().count().to_string().len()
        } else {
            0
        };

        let mut body = Vec::new();
        let mut interior = 0_usize;

        for (idx, raw) in content.lines().enumerate() {
            let base = if self.line_numbers {
                format!("{:>w$} │ {}", idx + 1, raw, w = digits)
            } else {
                raw.to_owned()
            };
            let rendered = base;
            interior = interior.max(rendered.len());
            body.push(rendered);
        }

        let header = format!(" File: {} ", path.display());
        interior = interior.max(header.len());
        interior = interior.max(self.width);

        let hr = "─".repeat(interior);

        writeln!(w, "┌{}┐", hr)?;
        writeln!(w, "│{}│", pad(&header, interior))?;
        writeln!(w, "├{}┤", hr)?;
        for line in body {
            writeln!(w, "│{}│", pad(&line, interior))?;
        }
        writeln!(w, "└{}┘", hr)?;
        Ok(())
    }
}

fn pad(s: &str, width: usize) -> String {
    if s.len() < width {
        let mut out = String::with_capacity(width);
        out.push_str(s);
        out.extend(std::iter::repeat(' ').take(width - s.len()));
        out
    } else {
        s.to_owned()
    }
}

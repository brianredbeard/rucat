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

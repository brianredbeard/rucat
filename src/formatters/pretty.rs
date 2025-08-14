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
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{self, Write};
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

static MODELINE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:vim|vi|ex)\s*:[^:]*:(?:ft|filetype|syntax)=([a-zA-Z0-9_.-]+)").unwrap()
});

/// Tries to find a syntax declaration in a Vim modeline.
fn find_syntax_from_modeline(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    // Check the first 5 lines.
    for line in lines.iter().take(5) {
        if let Some(caps) = MODELINE_RE.captures(line) {
            if let Some(syntax) = caps.get(1) {
                return Some(syntax.as_str().to_owned());
            }
        }
    }

    // Check the last 5 lines if the file is long enough.
    if line_count > 5 {
        for line in lines.iter().rev().take(5) {
            if let Some(caps) = MODELINE_RE.captures(line) {
                if let Some(syntax) = caps.get(1) {
                    return Some(syntax.as_str().to_owned());
                }
            }
        }
    }
    None
}

pub struct Pretty {
    pub line_numbers: bool,
    pub syntax_override: Option<String>,
}

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

impl Formatter for Pretty {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let syntax =
            // 1. Check for --pretty-syntax flag override.
            self.syntax_override
                .as_deref()
                .and_then(|s| SYNTAX_SET.find_syntax_by_token(s))
            // 2. Check for a Vim modeline in the content.
            .or_else(|| {
                find_syntax_from_modeline(content)
                    .and_then(|s| SYNTAX_SET.find_syntax_by_token(&s))
            })
            // 3. Fall back to the file extension.
            .or_else(|| {
                SYNTAX_SET.find_syntax_by_extension(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
            })
            // 4. Finally, use plain text.
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);

        if self.line_numbers {
            let digits = content.lines().count().to_string().len();
            for (idx, line) in content.lines().enumerate() {
                let escaped = match h.highlight_line(line, &SYNTAX_SET) {
                    Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], true),
                    Err(_) => line.to_string(), // Fallback to plain line on error
                };
                writeln!(w, "{:>w$} │ {}", idx + 1, escaped, w = digits)?;
            }
        } else {
            for line in content.lines() {
                let escaped = match h.highlight_line(line, &SYNTAX_SET) {
                    Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], true),
                    Err(_) => line.to_string(), // Fallback to plain line on error
                };
                writeln!(w, "{}", escaped)?;
            }
        }
        Ok(())
    }
}

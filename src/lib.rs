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
// Re-export everything tests need
pub mod cli;
pub mod formatters;

use crate::cli::OutputFormat;
use crate::formatters::{
    Formatter, ansi::Ansi, ascii::Ascii, markdown::Markdown, pretty::Pretty, utf8::Utf8, xml::Xml,
};

impl OutputFormat {
    pub fn into_formatter(
        &self,
        ansi_width: usize,
        utf8_width: usize,
        ln: bool,
        pretty_syntax: Option<&str>,
    ) -> Option<Box<dyn Formatter>> {
        match self {
            OutputFormat::Ansi => Some(Box::new(Ansi {
                width: ansi_width,
                line_numbers: ln,
            })),
            OutputFormat::Xml => Some(Box::new(Xml { line_numbers: ln })),
            OutputFormat::Markdown => Some(Box::new(Markdown { line_numbers: ln })),
            OutputFormat::Ascii => Some(Box::new(Ascii { line_numbers: ln })),
            OutputFormat::Utf8 => Some(Box::new(Utf8 {
                width: utf8_width,
                line_numbers: ln,
            })),
            OutputFormat::Pretty => Some(Box::new(Pretty {
                line_numbers: ln,
                syntax_override: pretty_syntax.map(String::from),
            })),
            OutputFormat::Json => None,
        }
    }
}

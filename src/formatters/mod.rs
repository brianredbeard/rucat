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
use crate::binary_display::BinaryFormatOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::metadata::FileMetadata;

pub trait Formatter {
    /// Writes a header, if any, for the formatted output.
    #[allow(unused_variables)]
    fn start(&mut self, w: &mut dyn Write) -> io::Result<()> {
        Ok(())
    }

    /// Writes the content to the given writer, applying formatting.
    ///
    /// # Errors
    ///
    /// Will return `Err` if it fails to write to the given writer.
    fn write(
        // Note: changed to &mut self
        &mut self,
        path: &Path,
        content: &str,
        metadata: Option<&FileMetadata>,
        options: &BinaryFormatOptions,
        w: &mut dyn Write,
    ) -> io::Result<()>;

    /// Writes a footer, if any, for the formatted output.
    #[allow(unused_variables)]
    fn finish(&mut self, w: &mut dyn Write) -> io::Result<()> {
        Ok(())
    }
}

pub mod ansi;
pub mod ascii; // simple “===” header
pub mod json;
pub mod markdown;
pub mod pretty;
pub mod utf8; // fancy UTF-8 borders
pub mod xml;

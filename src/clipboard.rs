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
use base64::Engine as _;
use std::io::{self, Write};

pub enum ClipboardProvider {
    Osc52,
    Osc5522,
}

impl ClipboardProvider {
    /// Copy content to the clipboard using the specified provider
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the output stream fails
    pub fn copy_to_clipboard(&self, content: &str, w: &mut dyn Write) -> io::Result<()> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());

        match self {
            Self::Osc52 => {
                write!(w, "\x1b]52;c;{encoded}\x07")?;
            }
            Self::Osc5522 => {
                write!(w, "\x1b]5522;{encoded}\x07")?;
            }
        }
        Ok(())
    }
}

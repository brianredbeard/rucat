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

//! Binary data display formats optimized for LLM comprehension

use clap::ValueEnum;
use serde::Deserialize;

/// Different formats for displaying binary data
#[derive(Debug, Clone, Copy, Default, ValueEnum, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BinaryOutputFormat {
    /// Hexadecimal with xxd-style layout (most LLM-friendly)
    #[default]
    Hexdump,
    /// Base64 encoding (very LLM-friendly, compact)
    Base64,
    /// Intel HEX format (structured for embedded systems)
    IntelHex,
    /// Raw hex bytes with separators
    RawHex,
}

#[derive(Debug, Clone, Copy)]
pub struct BinaryFormatOptions {
    pub show_binary: bool,
    pub format: BinaryOutputFormat,
    pub hex_width: usize,
    pub base64_width: usize,
}

/// Container for binary data with display formatting
pub struct BinaryDataFormatter<'a> {
    pub data: &'a [u8],
    pub name: &'a str,
}

impl<'a> BinaryDataFormatter<'a> {
    pub fn new(data: &'a [u8], name: &'a str) -> Self {
        Self { data, name }
    }

    /// Format binary data in a way that's most meaningful to LLMs
    pub fn format(
        &self,
        format: BinaryOutputFormat,
        hex_width: usize,
        base64_width: usize,
    ) -> String {
        match format {
            BinaryOutputFormat::Hexdump => self.format_hex_dump(hex_width, true, true),
            BinaryOutputFormat::Base64 => self.format_base64(Some(base64_width)),
            BinaryOutputFormat::IntelHex => {
                // Default base address, as it's not configurable via CLI yet
                self.format_intel_hex(0)
            }
            BinaryOutputFormat::RawHex => self.format_raw_hex(),
        }
    }

    /// xxd-style hex dump - excellent for LLM comprehension
    /// Format: 00000000: 1f8b 0800 0000 0000 0003 ec5d 7b7c 1445  ...........]{|.E
    fn format_hex_dump(
        &self,
        bytes_per_line: usize,
        show_offset: bool,
        show_ascii: bool,
    ) -> String {
        let mut output = String::new();

        if !self.data.is_empty() {
            output.push_str(&format!(
                "Binary data for '{}' ({} bytes):\n",
                self.name,
                self.data.len()
            ));
            output.push_str("Format: xxd-style hex dump (LLM-optimized)\n");
            output.push_str("# Each line shows: [offset]: [hex bytes] [ASCII representation]\n");
        }

        for (line_start, chunk) in self.data.chunks(bytes_per_line).enumerate() {
            let offset = line_start * bytes_per_line;

            if show_offset {
                output.push_str(&format!("{offset:08x}: "));
            }

            // Format hex bytes with spacing for readability
            let mut hex_part = String::new();
            for (i, byte) in chunk.iter().enumerate() {
                if i > 0 {
                    hex_part.push(' ');
                }
                hex_part.push_str(&format!("{byte:02x}"));
            }

            // Pad hex part to maintain alignment
            let expected_hex_width = if bytes_per_line > 0 {
                bytes_per_line * 3 - 1
            } else {
                0
            };
            output.push_str(&format!("{:width$}", hex_part, width = expected_hex_width));

            if show_ascii {
                output.push_str("  |");
                for byte in chunk {
                    let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                        *byte as char
                    } else {
                        '.'
                    };
                    output.push(ch);
                }
                output.push('|');
            }

            output.push('\n');
        }

        output
    }

    /// Base64 encoding - very LLM-friendly and compact
    fn format_base64(&self, line_width: Option<usize>) -> String {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let encoded = STANDARD.encode(self.data);
        let mut output = String::new();

        output.push_str(&format!(
            "Binary data for '{}' ({} bytes):\n",
            self.name,
            self.data.len()
        ));
        output.push_str("Format: Base64 encoding (LLM-optimized, compact)\n");
        output.push_str("# This format is highly readable by LLMs for decoding/processing\n");

        if let Some(width) = line_width {
            // Split into lines for better readability
            for chunk in encoded.as_bytes().chunks(width) {
                output.push_str(&String::from_utf8_lossy(chunk));
                output.push('\n');
            }
        } else {
            output.push_str(&encoded);
            output.push('\n');
        }

        output
    }

    /// Intel HEX format - structured and self-documenting
    fn format_intel_hex(&self, base_address: u16) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Binary data for '{}' ({} bytes):\n",
            self.name,
            self.data.len()
        ));
        output.push_str("Format: Intel HEX (structured, self-documenting)\n");
        output.push_str("# Format: :LLAAAATT[DD...]CC where:\n");
        output.push_str("# LL=length, AAAA=address, TT=type, DD=data, CC=checksum\n");

        for (i, chunk) in self.data.chunks(16).enumerate() {
            let address = base_address.wrapping_add((i * 16) as u16);
            let line = create_intel_hex_line(chunk, address);
            output.push_str(&line);
            output.push('\n');
        }

        // End of file record
        output.push_str(":00000001FF\n");
        output
    }

    /// Raw hex with no separators
    fn format_raw_hex(&self) -> String {
        let mut output = String::new();
        for byte in self.data {
            output.push_str(&format!("{byte:02x}"));
        }
        // No newline at the end for raw-hex
        output
    }
}

/// Create an Intel HEX line with checksum
fn create_intel_hex_line(data: &[u8], address: u16) -> String {
    let length = data.len() as u8;
    let addr_high = (address >> 8) as u8;
    let addr_low = (address & 0xFF) as u8;
    let record_type = 0x00u8; // Data record

    // Calculate checksum
    let mut checksum = length
        .wrapping_add(addr_high)
        .wrapping_add(addr_low)
        .wrapping_add(record_type);
    for &byte in data {
        checksum = checksum.wrapping_add(byte);
    }
    checksum = (!checksum).wrapping_add(1); // Two's complement

    let mut line = format!(":{length:02X}{address:04X}{record_type:02X}");
    for &byte in data {
        line.push_str(&format!("{byte:02X}"));
    }
    line.push_str(&format!("{checksum:02X}"));

    line
}

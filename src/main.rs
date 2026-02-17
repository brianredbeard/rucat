#![allow(clippy::multiple_crate_versions)]
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
use clap::Parser;
use rucat::binary_display::{BinaryDataFormatter, BinaryFormatOptions, BinaryOutputFormat};
use rucat::cli::{Args, OutputFormat};
#[cfg(feature = "clipboard")]
use rucat::clipboard::ClipboardProvider;
use rucat::metadata::FileMetadata;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const BINARY_PREVIEW_SIZE: usize = 160; // 10 lines of 16 bytes

/// Represents the content of a file, which can be either text or a binary preview.
enum FileContent {
    Text(String),
    Binary(Vec<u8>),
}

struct FormattingOptions<'a> {
    format: OutputFormat,
    line_numbers: bool,
    strip: usize,
    pretty_syntax: Option<&'a str>,
    ansi_width: usize,
    utf8_width: usize,
    stat: bool,
    binary_options: BinaryFormatOptions,
}

#[derive(Deserialize, Default)]
struct Config {
    format: Option<OutputFormat>,
    numbers: Option<bool>,
    strip: Option<usize>,
    ansi_width: Option<usize>,
    utf8_width: Option<usize>,
    pretty_syntax: Option<String>,
    stat: Option<bool>,
    show_binary_xattrs: Option<bool>,
    binary_format: Option<BinaryOutputFormat>,
    hex_width: Option<usize>,
    base64_width: Option<usize>,
}

fn load_config() -> Config {
    let config_path = if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg_config_home);
        path.push("rucat/config.toml");
        path
    } else if let Some(mut path) = dirs::config_dir() {
        path.push("rucat/config.toml");
        path
    } else {
        return Config::default();
    };
    if config_path.exists() {
        let content = fs::read_to_string(config_path).unwrap_or_default();
        return toml::from_str(&content).unwrap_or_default();
    }
    Config::default()
}

fn process_stdin(options: &FormattingOptions, writer: &mut dyn Write) -> anyhow::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let pseudo = PathBuf::from("-");

    let mut fmt = options
        .format
        .into_formatter(
            options.ansi_width,
            options.utf8_width,
            options.line_numbers,
            options.pretty_syntax,
        )
        .unwrap(); // Safe; all formats now return Some

    let disp = strip_components(&pseudo, options.strip);

    fmt.start(writer)?;
    fmt.write(&disp, &buf, None, &options.binary_options, writer)?;
    fmt.finish(writer)?;

    Ok(())
}

fn process_files(
    files: &[PathBuf],
    options: &FormattingOptions,
    writer: &mut dyn Write,
) -> anyhow::Result<()> {
    // Expand directories to individual files
    let mut paths = Vec::<PathBuf>::new();
    for p in files {
        if p.is_dir() {
            for entry in WalkDir::new(p)
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                paths.push(entry.into_path());
            }
        } else {
            paths.push(p.clone());
        }
    }

    // Safe to unwrap because all formats now return a formatter
    let mut fmt = options
        .format
        .into_formatter(
            options.ansi_width,
            options.utf8_width,
            options.line_numbers,
            options.pretty_syntax,
        )
        .unwrap();

    fmt.start(writer)?;

    for p in &paths {
        // A single "-" is a convention for stdin
        if p.to_string_lossy() == "-" {
            let mut stdin_buf = String::new();
            io::stdin().read_to_string(&mut stdin_buf)?;
            fmt.write(
                Path::new("-"),
                &stdin_buf,
                None, // No metadata for stdin
                &options.binary_options,
                writer,
            )?;
            continue; // Move to the next path
        }
        match read_file_content(p) {
            Ok(file_content) => {
                let metadata = if options.stat {
                    FileMetadata::extract(p)
                        .map_err(|e| {
                            eprintln!(
                                "Warning: Could not extract metadata for {}: {e}",
                                p.display()
                            );
                        })
                        .ok()
                } else {
                    None
                };

                let display_path = strip_components(p, options.strip);
                let content_to_format = match file_content {
                    FileContent::Text(text) => text,
                    FileContent::Binary(bytes) => {
                        // For binary files, we create a hex dump preview.
                        // The name/header for the hexdump is not needed here, as the
                        // main formatter (e.g., Markdown) provides the file header.
                        let formatter = BinaryDataFormatter::new(&bytes, "");
                        formatter.format(
                            options.binary_options.format,
                            options.binary_options.hex_width,
                            options.binary_options.base64_width,
                        )
                    }
                };
                fmt.write(
                    &display_path,
                    &content_to_format,
                    metadata.as_ref(),
                    &options.binary_options,
                    writer,
                )?;
            }
            Err(e) => writeln!(io::stderr(), "Error reading {}: {}", p.display(), e)?,
        }
    }

    fmt.finish(writer)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    let config = load_config();

    // Handle clipboard provider if copy flag is set
    #[cfg(feature = "clipboard")]
    let clipboard_provider = if args.copy {
        let provider = args.clipboard_provider_for_test.as_deref().map_or_else(
            ClipboardProvider::auto_detect,
            |provider_name| match provider_name {
                "osc52" => Some(ClipboardProvider::Osc52),
                "osc5522" => Some(ClipboardProvider::Osc5522),
                "native" => Some(ClipboardProvider::Native),
                _ => {
                    eprintln!("Error: Invalid test provider '{provider_name}'");
                    std::process::exit(1);
                }
            },
        );
        if provider.is_none() {
            eprintln!(
                "Error: Failed to initialize clipboard: no suitable provider found for your environment."
            );
            std::process::exit(1);
        }
        provider
    } else {
        None
    };

    // Merge settings: CLI > Config File > Default
    let format = args
        .format
        .or(config.format)
        .unwrap_or(OutputFormat::Markdown);
    let line_numbers = args.line_numbers || config.numbers.unwrap_or(false);
    let strip = args.strip.or(config.strip).unwrap_or(0);
    let ansi_width = args.ansi_width.or(config.ansi_width).unwrap_or(80);
    let utf8_width = args.utf8_width.or(config.utf8_width).unwrap_or(80);
    let pretty_syntax = args.pretty_syntax.or(config.pretty_syntax);
    let stat = args.stat || config.stat.unwrap_or(false);
    let show_binary = args.show_binary_xattrs || config.show_binary_xattrs.unwrap_or(false);
    let binary_format = args
        .binary_format
        .or(config.binary_format)
        .unwrap_or_default();
    let hex_width = args.hex_width.or(config.hex_width).unwrap_or(16).max(1);
    let base64_width = args
        .base64_width
        .or(config.base64_width)
        .unwrap_or(76)
        .max(1);

    let binary_options = BinaryFormatOptions {
        show_binary,
        format: binary_format,
        hex_width,
        base64_width,
    };

    let formatting_options = FormattingOptions {
        format,
        line_numbers,
        strip,
        pretty_syntax: pretty_syntax.as_deref(),
        ansi_width,
        utf8_width,
        stat,
        binary_options,
    };

    // If the user passed -0/--null, pull a NUL-separated list of paths from stdin
    if args.null_sep {
        let mut bytes = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        for part in bytes.split(|b| *b == 0) {
            if part.is_empty() {
                continue;
            }
            #[cfg(unix)]
            let pb = PathBuf::from(std::ffi::OsStr::from_bytes(part));
            #[cfg(not(unix))]
            let pb = PathBuf::from(String::from_utf8_lossy(part).to_string());
            args.files.push(pb);
        }
    }

    // If copying to clipboard, buffer all output first. Otherwise, write directly to stdout.
    #[cfg(feature = "clipboard")]
    if let Some(provider) = clipboard_provider {
        let mut buffer = Vec::new();
        if args.files.is_empty() && !args.null_sep {
            process_stdin(&formatting_options, &mut buffer)?;
        } else {
            process_files(&args.files, &formatting_options, &mut buffer)?;
        }

        // Write buffer to stdout
        io::stdout().write_all(&buffer)?;

        // Pass buffer to clipboard provider
        let content = String::from_utf8_lossy(&buffer);
        provider.copy_to_clipboard(&content, &mut io::stdout())?;
    } else {
        // Not using clipboard, write directly to stdout
        let mut stdout = io::stdout();
        if args.files.is_empty() && !args.null_sep {
            process_stdin(&formatting_options, &mut stdout)?;
        } else {
            process_files(&args.files, &formatting_options, &mut stdout)?;
        }
    }

    #[cfg(not(feature = "clipboard"))]
    {
        let mut stdout = io::stdout();
        if args.files.is_empty() && !args.null_sep {
            process_stdin(&formatting_options, &mut stdout)?;
        } else {
            process_files(&args.files, &formatting_options, &mut stdout)?;
        }
    }

    Ok(())
}

fn read_file_content(p: &PathBuf) -> anyhow::Result<FileContent> {
    // First, try to read as a string. This is fast and handles the common case.
    match std::fs::read_to_string(p) {
        Ok(content) => Ok(FileContent::Text(content)),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            // If it's not valid UTF-8, it's a binary file.
            // Read a small preview instead of the whole file.
            let file = File::open(p)?;
            let mut buffer = Vec::with_capacity(BINARY_PREVIEW_SIZE);
            file.take(BINARY_PREVIEW_SIZE as u64)
                .read_to_end(&mut buffer)?;

            Ok(FileContent::Binary(buffer))
        }
        Err(e) => {
            // For other errors (e.g., permission denied), propagate them.
            Err(e.into())
        }
    }
}

fn strip_components(p: &Path, n: usize) -> PathBuf {
    let parts: Vec<_> = p.iter().collect();
    if parts.is_empty() {
        return p.to_path_buf();
    }

    // we must keep at least the filename
    let start = if parts.len() > 1 {
        std::cmp::min(n, parts.len() - 1)
    } else {
        0
    };
    parts[start..].iter().collect()
}

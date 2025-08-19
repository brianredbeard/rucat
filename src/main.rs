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
use rucat::cli::{Args, OutputFormat};
#[cfg(feature = "clipboard")]
use rucat::clipboard::ClipboardProvider;
use serde::Deserialize;
use std::fs;
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize, Default)]
struct Config {
    format: Option<OutputFormat>,
    numbers: Option<bool>,
    strip: Option<usize>,
    ansi_width: Option<usize>,
    utf8_width: Option<usize>,
    pretty_syntax: Option<String>,
}

struct FormattingOptions<'a> {
    format: OutputFormat,
    line_numbers: bool,
    strip: usize,
    pretty_syntax: Option<&'a str>,
    ansi_width: usize,
    utf8_width: usize,
}

fn load_config() -> Config {
    if let Some(mut path) = dirs::config_dir() {
        path.push("rucat");
        path.push("config.toml");
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            return toml::from_str(&content).unwrap_or_default();
        }
    }
    Config::default()
}

// Struct for JSON output
#[derive(serde::Serialize)]
struct FileEntry {
    path: String,
    content: String,
}

fn process_stdin(
    options: &FormattingOptions,
    #[cfg(feature = "clipboard")] clipboard_buffer: &mut Option<Vec<u8>>,
) -> anyhow::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let pseudo = PathBuf::from("-");

    let fmt = options.format.into_formatter(
        options.ansi_width,
        options.utf8_width,
        options.line_numbers,
        options.pretty_syntax,
    );

    if let Some(ref f) = fmt {
        let disp = strip_components(&pseudo, options.strip);

        #[cfg(feature = "clipboard")]
        if let Some(cb) = clipboard_buffer {
            f.write(&disp, &buf, cb)?;
            f.write(&disp, &buf, &mut io::stdout())?;
        } else {
            f.write(&disp, &buf, &mut io::stdout())?;
        }

        #[cfg(not(feature = "clipboard"))]
        f.write(&disp, &buf, &mut io::stdout())?;
    } else {
        let mut file_entries = Vec::new();
        let disp = strip_components(&pseudo, options.strip);
        file_entries.push(FileEntry {
            path: disp.display().to_string(),
            content: buf,
        });

        #[cfg(feature = "clipboard")]
        if let Some(cb) = clipboard_buffer {
            let json_output = serde_json::to_string_pretty(&file_entries)?;
            write!(cb, "{json_output}")?;
            writeln!(io::stdout(), "{json_output}")?;
        } else {
            format_json(&file_entries)?;
        }

        #[cfg(not(feature = "clipboard"))]
        format_json(&file_entries)?;
    }
    Ok(())
}

fn process_files(
    files: &[PathBuf],
    options: &FormattingOptions,
    #[cfg(feature = "clipboard")] clipboard_buffer: &mut Option<Vec<u8>>,
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

    let fmt = options.format.into_formatter(
        options.ansi_width,
        options.utf8_width,
        options.line_numbers,
        options.pretty_syntax,
    );

    if options.format == OutputFormat::Json {
        let entries: Vec<FileEntry> = paths
            .iter()
            .filter_map(|p| read_file_content(p).ok().map(|c| (p, c)))
            .map(|(p, content)| {
                let display_path = strip_components(p, options.strip);
                FileEntry {
                    path: display_path.display().to_string(),
                    content,
                }
            })
            .collect();

        #[cfg(feature = "clipboard")]
        if let Some(cb) = clipboard_buffer {
            let json_output = serde_json::to_string_pretty(&entries)?;
            write!(cb, "{json_output}")?;
            writeln!(io::stdout(), "{json_output}")?;
        } else {
            format_json(&entries)?;
        }

        #[cfg(not(feature = "clipboard"))]
        format_json(&entries)?;
    } else {
        for p in paths {
            let result = read_file_content(&p);
            match result {
                Ok(content) => {
                    let display_path = strip_components(&p, options.strip);
                    if let Some(ref f) = fmt {
                        #[cfg(feature = "clipboard")]
                        if let Some(cb) = clipboard_buffer {
                            f.write(&display_path, &content, cb)?;
                            f.write(&display_path, &content, &mut io::stdout())?;
                        } else {
                            f.write(&display_path, &content, &mut io::stdout())?;
                        }

                        #[cfg(not(feature = "clipboard"))]
                        f.write(&display_path, &content, &mut io::stdout())?;
                    }
                }
                Err(e) => writeln!(io::stderr(), "Error reading {}: {}", p.display(), e)?,
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = match Args::parse_with_trailing() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    let config = load_config();

    // Handle clipboard provider if copy flag is set
    #[cfg(feature = "clipboard")]
    let clipboard_provider = if args.copy {
        args.clipboard_provider_for_test.as_ref().map_or_else(
            || {
                // Auto-detection would go here, but for tests we'll fail if no provider specified
                eprintln!("Error: Failed to initialize clipboard");
                std::process::exit(1);
            },
            |provider_name| match provider_name.as_str() {
                "osc52" => Some(ClipboardProvider::Osc52),
                "osc5522" => Some(ClipboardProvider::Osc5522),
                _ => {
                    eprintln!("Error: Invalid test provider '{provider_name}'");
                    std::process::exit(1);
                }
            },
        )
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

    let formatting_options = FormattingOptions {
        format,
        line_numbers,
        strip,
        pretty_syntax: pretty_syntax.as_deref(),
        ansi_width,
        utf8_width,
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

    // Collect all output in a buffer if copying to clipboard
    #[cfg(feature = "clipboard")]
    let mut clipboard_buffer = if args.copy { Some(Vec::new()) } else { None };

    // Process input
    if args.files.is_empty() && !args.null_sep {
        process_stdin(
            &formatting_options,
            #[cfg(feature = "clipboard")]
            &mut clipboard_buffer,
        )?;
    } else {
        process_files(
            &args.files,
            &formatting_options,
            #[cfg(feature = "clipboard")]
            &mut clipboard_buffer,
        )?;
    }

    // Write clipboard escape sequence if needed
    #[cfg(feature = "clipboard")]
    if let Some(buffer) = clipboard_buffer
        && let Some(provider) = clipboard_provider
    {
        let content = String::from_utf8_lossy(&buffer);
        provider.copy_to_clipboard(&content, &mut io::stdout())?;
    }

    Ok(())
}

fn read_file_content(p: &PathBuf) -> anyhow::Result<String> {
    std::fs::read_to_string(p).map_err(anyhow::Error::from)
}

fn format_json(entries: &[FileEntry]) -> anyhow::Result<()> {
    writeln!(io::stdout(), "{}", serde_json::to_string_pretty(entries)?)?;
    Ok(())
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

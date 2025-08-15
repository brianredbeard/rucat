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
use rucat::cli::{Args, OutputFormat};
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

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    let config = load_config();

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

    // If no files are specified, read from stdin
    if args.files.is_empty() && !args.null_sep {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let pseudo = PathBuf::from("-");
        args.files.push(pseudo.clone());
        let fmt = format.into_formatter(
            ansi_width,
            utf8_width,
            line_numbers,
            pretty_syntax.as_deref(),
        );
        if let Some(ref f) = fmt {
            let disp = strip_components(&pseudo, strip);
            f.write(&disp, &buf, &mut io::stdout())?;
        } else {
            let mut file_entries = Vec::new();
            let disp = strip_components(&pseudo, strip);
            file_entries.push(FileEntry {
                path: disp.display().to_string(),
                content: buf,
            });
            return format_json(&file_entries);
        }
    } else {
        // -------- expand directories to individual files --------
        let mut paths = Vec::<PathBuf>::new();
        for p in &args.files {
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

        let fmt = format.into_formatter(
            ansi_width,
            utf8_width,
            line_numbers,
            pretty_syntax.as_deref(),
        );
        if let OutputFormat::Json = format {
            let entries: Vec<FileEntry> = paths
                .iter()
                .filter_map(|p| read_file_content(p).ok().map(|c| (p, c)))
                .map(|(p, content)| {
                    let display_path = strip_components(p, strip);
                    FileEntry {
                        path: display_path.display().to_string(),
                        content,
                    }
                })
                .collect();
            format_json(&entries)?;
        } else {
            for p in paths {
                let result = read_file_content(&p);
                match result {
                    Ok(content) => {
                        let display_path = strip_components(&p, strip);
                        if let Some(ref f) = fmt {
                            f.write(&display_path, &content, &mut io::stdout())?;
                        }
                    }
                    Err(e) => writeln!(io::stderr(), "Error reading {}: {}", p.display(), e)?,
                }
            }
        }
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

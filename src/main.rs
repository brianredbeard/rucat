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
use std::path::{Path, PathBuf};
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use serde::{Serialize, Deserialize};
use rucat::formatters::{
    Formatter, ansi::Ansi, xml::Xml, markdown::Markdown, ascii::Ascii, utf8::Utf8,
};
use rayon::prelude::*;
use walkdir::WalkDir;
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files to process
    #[arg(value_hint = clap::ValueHint::FilePath, num_args = 0..)]
    files: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,

    /// Width for ANSI formatting (excluding borders)
    #[arg(long, default_value_t = 80)]
    ansi_width: usize,

    /// Add a gutter with line numbers
    #[arg(short = 'n', long = "numbers")]
    line_numbers: bool,

    /// Read NUL-terminated file list from STDIN (like `xargs -0`)
    #[arg(short = '0', long = "null")]
    null_sep: bool,

    /// Remove N leading path components when printing filenames
    #[arg(long = "strip", default_value_t = 0)]
    strip: usize,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// ANSI box drawing characters
    Ansi,
    /// XML format
    Xml,
    /// JSON format
    Json,
    /// Markdown code blocks
    Markdown,
    /// Simple ASCII header
    Ascii,
    /// Fancy UTF-8 box drawing
    Utf8,
}

impl OutputFormat {
    fn into_formatter(&self, width: usize, ln: bool) -> Option<Box<dyn Formatter>> {
        match self {
            OutputFormat::Ansi     => Some(Box::new(Ansi     { width,          line_numbers: ln })),
            OutputFormat::Xml      => Some(Box::new(Xml      {                line_numbers: ln })),
            OutputFormat::Markdown => Some(Box::new(Markdown {                line_numbers: ln })),
            OutputFormat::Ascii    => Some(Box::new(Ascii    {                line_numbers: ln })),
            OutputFormat::Utf8     => Some(Box::new(Utf8     { width,          line_numbers: ln })),
            OutputFormat::Json     => None,
        }
    }
}

// Struct for JSON output
#[derive(Serialize, Deserialize)]
struct FileEntry {
    path: String,
    content: String,
}

fn main() -> Result<()> {
    let mut args = Args::parse();

    // If the user passed -0/--null, pull a NUL-separated list of paths from stdin
    if args.null_sep {
        let mut bytes = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        for part in bytes.split(|b| *b == 0) {
            if part.is_empty() { continue }
            #[cfg(unix)]
            let pb = PathBuf::from(std::ffi::OsStr::from_bytes(part));
            #[cfg(not(unix))]
            let pb = PathBuf::from(String::from_utf8_lossy(part).to_string());
            args.files.push(pb);
        }
    }

    if args.files.is_empty() && !args.null_sep {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let pseudo = PathBuf::from("-");
        args.files.push(pseudo.clone());
        // We'll use the buffer below
        let fmt = args.format.into_formatter(args.ansi_width, args.line_numbers);
        if let Some(ref f) = fmt {
            let disp = strip_components(&pseudo, args.strip);
            f.write(&disp, &buf, &mut io::stdout())?;
        } else {
            let mut file_entries = Vec::new();
            let disp = strip_components(&pseudo, args.strip);
            file_entries.push(FileEntry {
                path: disp.display().to_string(),
                content: buf,
            });
            format_json(&file_entries)?;
        }
        return Ok(());
    }

    // -------- expand directories to individual files --------
    let mut paths = Vec::<PathBuf>::new();
    for p in &args.files {
        if p.is_dir() {
            for entry in WalkDir::new(p)
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

    let fmt = args.format.into_formatter(args.ansi_width, args.line_numbers);

    let results: Vec<_> = paths
        .par_iter()
        .map(|p| (p.clone(), read_file_content(p)))
        .collect();

    let mut file_entries = Vec::new();

    for (file_path, res) in results {
        match res {
            Ok(content) => {
                let display_path = strip_components(&file_path, args.strip);
                if let Some(ref f) = fmt {
                    f.write(&display_path, &content, &mut io::stdout())?;
                } else {
                    file_entries.push(FileEntry {
                        path: display_path.display().to_string(),
                        content,
                    });
                }
            }
            Err(e) => eprintln!("Error reading {}: {}", file_path.display(), e),
        }
    }

    if let OutputFormat::Json = args.format {
        format_json(&file_entries)?;
    }

    Ok(())
}

fn read_file_content(p: &PathBuf) -> Result<String> {
    std::fs::read_to_string(p)
        .with_context(|| format!("Failed to read {}", p.display()))
}

fn format_json(entries: &[FileEntry]) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(entries)?);
    Ok(())
}

fn strip_components(p: &Path, n: usize) -> PathBuf {
    // collect all normal components (root is excluded on Unix)
    let parts: Vec<_> = p.iter().collect();
    if parts.is_empty() {
        return p.to_path_buf();
    }

    // we must keep at least the filename → trim at most len-1 components
    let start = std::cmp::min(n, parts.len() - 1);
    parts[start..].iter().collect()
}

mod formatters;

use clap::Parser;
use std::path::PathBuf;
use std::io::{self, Read};
use serde::{Serialize, Deserialize};
use formatters::{Formatter, ansi::Ansi, xml::Xml, markdown::Markdown};
use rayon::prelude::*;
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
    /// Plain UTF-8 with header
    Utf8,
}

impl OutputFormat {
    fn into_formatter(&self, width: usize) -> Option<Box<dyn Formatter>> {
        match self {
            OutputFormat::Ansi     => Some(Box::new(Ansi { width })),
            OutputFormat::Xml      => Some(Box::new(Xml)),
            OutputFormat::Markdown => Some(Box::new(Markdown)),
            OutputFormat::Utf8     => Some(Box::new(Utf8)),
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

    if args.files.is_empty() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let pseudo = PathBuf::from("-");
        args.files.push(pseudo.clone());
        // We'll use the buffer below
        let fmt = args.format.into_formatter(args.ansi_width);
        if let Some(ref f) = fmt {
            f.write(&pseudo, &buf, &mut io::stdout())?;
        } else {
            let mut file_entries = Vec::new();
            file_entries.push(FileEntry {
                path: pseudo.display().to_string(),
                content: buf,
            });
            format_json(&file_entries)?;
        }
        return Ok(());
    }

    let fmt = args.format.into_formatter(args.ansi_width);

    let results: Vec<_> = args.files
        .par_iter()
        .map(|p| (p.clone(), read_file_content(p)))
        .collect();

    let mut file_entries = Vec::new();

    for (file_path, res) in results {
        match res {
            Ok(content) => {
                if let Some(ref f) = fmt {
                    f.write(&file_path, &content, &mut io::stdout())?;
                } else {
                    file_entries.push(FileEntry {
                        path: file_path.display().to_string(),
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

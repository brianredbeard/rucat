use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Files to process
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub files: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Width for ANSI formatting (excluding borders)
    #[arg(long)]
    pub ansi_width: Option<usize>,

    /// Width for UTF8 formatting (excluding borders)
    #[arg(long)]
    pub utf8_width: Option<usize>,

    /// Add a gutter with line numbers
    #[arg(short = 'n', long = "numbers")]
    pub line_numbers: bool,

    /// Read NUL-terminated file list from STDIN (like `xargs -0`)
    #[arg(short = '0', long = "null")]
    pub null_sep: bool,

    /// Remove N leading path components when printing filenames
    #[arg(long, value_name = "N")]
    pub strip: Option<usize>,

    /// Explicitly set the syntax for the 'pretty' formatter
    #[arg(long)]
    pub pretty_syntax: Option<String>,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
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
    /// Pretty-printed with syntax highlighting
    Pretty,
}

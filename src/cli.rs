use crate::binary_display::BinaryOutputFormat;
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

fn validate_nonzero_usize(s: &str) -> Result<usize, String> {
    let val: usize = s
        .parse()
        .map_err(|_| "must be a valid number".to_string())?;
    if val == 0 {
        Err("must be at least 1".to_string())
    } else {
        Ok(val)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
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

    /// Include file metadata (e.g., extended attributes)
    #[arg(long)]
    pub stat: bool,

    /// Show binary extended attributes (normally hidden)
    #[arg(long)]
    pub show_binary_xattrs: bool,

    /// Format for binary data display
    #[arg(long, value_enum)]
    pub binary_format: Option<BinaryOutputFormat>,

    /// Bytes per line for hex display
    #[arg(long, value_parser = validate_nonzero_usize)]
    pub hex_width: Option<usize>,

    /// Characters per line for base64
    #[arg(long, value_parser = validate_nonzero_usize)]
    pub base64_width: Option<usize>,

    /// Copy output to the system clipboard
    #[cfg(feature = "clipboard")]
    #[arg(short, long)]
    pub copy: bool,

    /// FOR TESTING ONLY: Force a specific clipboard provider
    #[cfg(feature = "clipboard")]
    #[arg(long, hide = true)]
    pub clipboard_provider_for_test: Option<String>,

    /// Files to process
    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,
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

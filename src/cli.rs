use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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

    /// Copy output to the system clipboard
    #[cfg(feature = "clipboard")]
    #[arg(short, long)]
    pub copy: bool,

    /// FOR TESTING ONLY: Force a specific clipboard provider
    #[cfg(feature = "clipboard")]
    #[arg(long, hide = true)]
    pub clipboard_provider_for_test: Option<String>,

    /// Files and trailing options (allows -c after files)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_name = "FILES...", help = "Files to process (options like -c/--copy can appear after files)")]
    pub raw_args: Vec<String>,

    /// Files to process (populated from raw_args)
    #[arg(skip)]
    pub files: Vec<PathBuf>,
}

impl Args {
    /// Parse arguments, handling trailing flags after files
    pub fn parse_with_trailing() -> Result<Self, String> {
        let mut args = Self::parse();
        
        // Process raw_args to separate files from trailing flags
        let mut files = Vec::new();
        let mut i = 0;
        
        while i < args.raw_args.len() {
            let arg = &args.raw_args[i];
            
            // Check for trailing flags
            match arg.as_str() {
                "-c" | "--copy" => {
                    #[cfg(feature = "clipboard")]
                    {
                        args.copy = true;
                    }
                    #[cfg(not(feature = "clipboard"))]
                    {
                        return Err(format!("The '{}' flag requires the 'clipboard' feature to be enabled", arg));
                    }
                }
                "-n" | "--numbers" => {
                    args.line_numbers = true;
                }
                "-0" | "--null" => {
                    args.null_sep = true;
                }
                // Handle format flags
                "-f" | "--format" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    let format_str = &args.raw_args[i];
                    match format_str.as_str() {
                        "ansi" => args.format = Some(OutputFormat::Ansi),
                        "xml" => args.format = Some(OutputFormat::Xml),
                        "json" => args.format = Some(OutputFormat::Json),
                        "markdown" => args.format = Some(OutputFormat::Markdown),
                        "ascii" => args.format = Some(OutputFormat::Ascii),
                        "utf8" => args.format = Some(OutputFormat::Utf8),
                        "pretty" => args.format = Some(OutputFormat::Pretty),
                        _ => return Err(format!("Invalid format '{}'. Valid formats are: ansi, xml, json, markdown, ascii, utf8, pretty", format_str)),
                    }
                }
                "--strip" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    let strip_str = &args.raw_args[i];
                    match strip_str.parse::<usize>() {
                        Ok(n) => args.strip = Some(n),
                        Err(_) => return Err(format!("Invalid value '{}' for --strip, expected a number", strip_str)),
                    }
                }
                "--ansi-width" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    let width_str = &args.raw_args[i];
                    match width_str.parse::<usize>() {
                        Ok(n) => args.ansi_width = Some(n),
                        Err(_) => return Err(format!("Invalid value '{}' for --ansi-width, expected a number", width_str)),
                    }
                }
                "--utf8-width" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    let width_str = &args.raw_args[i];
                    match width_str.parse::<usize>() {
                        Ok(n) => args.utf8_width = Some(n),
                        Err(_) => return Err(format!("Invalid value '{}' for --utf8-width, expected a number", width_str)),
                    }
                }
                "--pretty-syntax" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    args.pretty_syntax = Some(args.raw_args[i].clone());
                }
                #[cfg(feature = "clipboard")]
                "--clipboard-provider-for-test" => {
                    if i + 1 >= args.raw_args.len() {
                        return Err(format!("The '{}' flag requires a value", arg));
                    }
                    i += 1;
                    args.clipboard_provider_for_test = Some(args.raw_args[i].clone());
                }
                _ if arg.starts_with('-') => {
                    // Unknown flag
                    return Err(format!(
                        "Unknown flag '{}'. Use --help to see available options.\n\
                        Note: Flags can appear before or after files.",
                        arg
                    ));
                }
                _ => {
                    // It's a file
                    files.push(PathBuf::from(arg));
                }
            }
            i += 1;
        }
        
        args.files = files;
        Ok(args)
    }
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

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
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "FILES...",
        help = "Files to process (options like -c/--copy can appear after files)"
    )]
    pub raw_args: Vec<String>,

    /// Files to process (populated from `raw_args`)
    #[arg(skip)]
    pub files: Vec<PathBuf>,
}

impl Args {
    /// Parse arguments, handling trailing flags after files
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An unknown flag is encountered
    /// - A flag requiring a value is missing its value
    /// - A flag value is invalid (e.g., non-numeric value for numeric flags)
    /// - The clipboard feature is not enabled but clipboard flags are used
    pub fn parse_with_trailing() -> Result<Self, String> {
        let mut args = Self::parse();

        // Clone raw_args to avoid borrow checker issues
        let raw_args = args.raw_args.clone();

        // Process raw_args to separate files from trailing flags
        let mut files = Vec::new();
        let mut i = 0;

        while i < raw_args.len() {
            // Clone the arg to avoid borrow issues
            let arg = raw_args[i].clone();

            // Try to parse as flag, if not, treat as file
            if !Self::parse_flag(&mut args, &raw_args, &mut i)? {
                // Not a flag, treat as file
                files.push(PathBuf::from(arg));
                i += 1;
            }
            // Otherwise flag was parsed, i was already incremented
        }

        args.files = files;
        Ok(args)
    }

    /// Parse a single flag from the arguments
    /// Returns true if a flag was parsed, false if the argument is not a flag
    fn parse_flag(args: &mut Self, raw_args: &[String], i: &mut usize) -> Result<bool, String> {
        let arg = &raw_args[*i];

        match arg.as_str() {
            "-c" | "--copy" => Self::handle_copy_flag(args, arg, i),
            "-n" | "--numbers" => {
                args.line_numbers = true;
                *i += 1;
                Ok(true)
            }
            "-0" | "--null" => {
                args.null_sep = true;
                *i += 1;
                Ok(true)
            }
            "-f" | "--format" => Self::handle_format_flag(args, raw_args, i),
            "--strip" => Self::handle_numeric_flag(raw_args, i, |n| args.strip = Some(n), "strip"),
            "--ansi-width" => {
                Self::handle_numeric_flag(raw_args, i, |n| args.ansi_width = Some(n), "ansi-width")
            }
            "--utf8-width" => {
                Self::handle_numeric_flag(raw_args, i, |n| args.utf8_width = Some(n), "utf8-width")
            }
            "--pretty-syntax" => {
                Self::handle_string_flag(raw_args, i, |s| args.pretty_syntax = Some(s))
            }
            #[cfg(feature = "clipboard")]
            "--clipboard-provider-for-test" => Self::handle_string_flag(raw_args, i, |s| {
                args.clipboard_provider_for_test = Some(s);
            }),
            _ if arg.starts_with('-') => Err(format!(
                "Unknown flag '{arg}'. Use --help to see available options.\n\
                    Note: Flags can appear before or after files."
            )),
            _ => Ok(false),
        }
    }

    #[cfg(feature = "clipboard")]
    #[allow(clippy::missing_const_for_fn)] // False positive: function mutates args
    #[allow(clippy::unnecessary_wraps)] // Must match signature of other cfg branch
    fn handle_copy_flag(args: &mut Self, _arg: &str, i: &mut usize) -> Result<bool, String> {
        args.copy = true;
        *i += 1;
        Ok(true)
    }

    #[cfg(not(feature = "clipboard"))]
    fn handle_copy_flag(_args: &mut Self, arg: &str, _i: &mut usize) -> Result<bool, String> {
        Err(format!(
            "The '{arg}' flag requires the 'clipboard' feature to be enabled"
        ))
    }

    fn handle_format_flag(
        args: &mut Self,
        raw_args: &[String],
        i: &mut usize,
    ) -> Result<bool, String> {
        let arg = &raw_args[*i];
        if *i + 1 >= raw_args.len() {
            return Err(format!("The '{arg}' flag requires a value"));
        }
        *i += 1;
        let format_str = &raw_args[*i];
        match format_str.as_str() {
            "ansi" => args.format = Some(OutputFormat::Ansi),
            "xml" => args.format = Some(OutputFormat::Xml),
            "json" => args.format = Some(OutputFormat::Json),
            "markdown" => args.format = Some(OutputFormat::Markdown),
            "ascii" => args.format = Some(OutputFormat::Ascii),
            "utf8" => args.format = Some(OutputFormat::Utf8),
            "pretty" => args.format = Some(OutputFormat::Pretty),
            _ => {
                return Err(format!(
                    "Invalid format '{format_str}'. Valid formats are: ansi, xml, json, markdown, ascii, utf8, pretty"
                ));
            }
        }
        *i += 1;
        Ok(true)
    }

    fn handle_numeric_flag<F>(
        raw_args: &[String],
        i: &mut usize,
        setter: F,
        flag_name: &str,
    ) -> Result<bool, String>
    where
        F: FnOnce(usize),
    {
        let arg = &raw_args[*i];
        if *i + 1 >= raw_args.len() {
            return Err(format!("The '{arg}' flag requires a value"));
        }
        *i += 1;
        let value_str = &raw_args[*i];
        value_str.parse::<usize>().map_or_else(
            |_| {
                Err(format!(
                    "Invalid value '{value_str}' for --{flag_name}, expected a number"
                ))
            },
            |n| {
                setter(n);
                *i += 1;
                Ok(true)
            },
        )
    }

    fn handle_string_flag<F>(raw_args: &[String], i: &mut usize, setter: F) -> Result<bool, String>
    where
        F: FnOnce(String),
    {
        let arg = &raw_args[*i];
        if *i + 1 >= raw_args.len() {
            return Err(format!("The '{arg}' flag requires a value"));
        }
        *i += 1;
        setter(raw_args[*i].clone());
        *i += 1;
        Ok(true)
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

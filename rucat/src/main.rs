use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Read};
use serde::{Serialize, Deserialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files to process
    #[arg(required = true)]
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
}

// Struct for JSON output
#[derive(Serialize, Deserialize)]
struct FileEntry {
    path: String,
    content: String,
}


fn main() -> io::Result<()> {
    let args = Args::parse();

    // Collect all file entries for JSON output
    let mut file_entries = Vec::new();

    for file_path in args.files {
        match read_file_content(&file_path) {
            Ok(content) => {
                match args.format {
                    OutputFormat::Ansi => format_ansi(&file_path, &content, args.ansi_width),
                    OutputFormat::Xml => format_xml(&file_path, &content),
                    OutputFormat::Json => {
                        // Collect for later printing as a single JSON array
                        file_entries.push(FileEntry {
                            path: file_path.display().to_string(),
                            content,
                        });
                    },
                    OutputFormat::Markdown => format_markdown(&file_path, &content),
                }
            }
            Err(err) => {
                eprintln!("Error reading file {}: {}", file_path.display(), err);
                // Continue processing other files
            }
        }
    }

    // Print JSON output if requested
    if let OutputFormat::Json = args.format {
        format_json(&file_entries)?; // Pass the vector of entries
    }

    Ok(())
}

fn read_file_content(file_path: &PathBuf) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// --- Formatting Functions ---

fn format_ansi(file_path: &PathBuf, content: &str, width: usize) {
    let horizontal_line = "─".repeat(width);
    let file_info = format!(" File: {}", file_path.display());
    // Ensure padding doesn't underflow if file_info is longer than width
    let file_info_padding_len = width.saturating_sub(file_info.len() + 1); // +1 for the leading space
    let file_info_padded = format!(" {}{}", file_info, " ".repeat(file_info_padding_len));

    // Top border
    println!("┬{}─", horizontal_line);

    // File info line
    println!("│{}│", file_info_padded);

    // Separator
    println!("┼{}─", horizontal_line);

    // Content lines
    for line in content.lines() {
        // Ensure padding doesn't underflow
        let line_padding_len = width.saturating_sub(line.len() + 1); // +1 for the leading space
        let line_padded = format!(" {}{}", line, " ".repeat(line_padding_len));
        println!("│{}│", line_padded);
    }

    // Bottom border
    println!("┴{}─", horizontal_line);
}

fn format_xml(file_path: &PathBuf, content: &str) {
    // Basic XML escaping
    let escaped_content = content.replace("&", "&amp;")
                                 .replace("<", "&lt;")
                                 .replace(">", "&gt;")
                                 .replace("\"", "&quot;")
                                 .replace("'", "&apos;");
    println!("<file path=\"{}\">{}</file>", file_path.display(), escaped_content);
}

// Modified format_json to take a vector and print the whole array
fn format_json(entries: &Vec<FileEntry>) -> io::Result<()> {
    // Use serde_json to serialize the vector of FileEntry structs
    let json_output = serde_json::to_string_pretty(entries)?; // Use pretty for readability
    println!("{}", json_output);
    Ok(())
}

fn format_markdown(file_path: &PathBuf, content: &str) {
    let extension = file_path.extension()
                             .and_then(|s| s.to_str())
                             .unwrap_or(""); // Get file extension for language hint

    println!("---\nFile: {}\n---", file_path.display());
    // Use the extension as the language hint for the code block
    println!("```{}\n{}\n```", extension, content);
}

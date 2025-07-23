# `rucat` - A versatile `cat` clone

`rucat` is a modern, feature-rich replacement for the classic `cat` utility, written in Rust. It goes beyond simple file concatenation, offering multiple output formats, line numbering, and syntax-aware formatting, making it an ideal tool for developers, system administrators, and anyone working with code or text files in the terminal.

## Features

-   **Multiple Output Formats**: Display content in various styles, including:
    -   `ansi`: Nicely formatted with borders (width-configurable via `--ansi-width`).
    -   `utf8`: Fancy UTF-8 box-drawing borders (width-configurable via `--ansi-width`).
    -   `markdown`: GitHub-flavored Markdown code blocks, with automatic language detection from the file extension.
    -   `ascii`: Simple `=== file.txt ===` headers for easy separation.
    -   `xml`: Structured XML output with file and line metadata.
    -   `json`: A clean JSON array of file entries, perfect for scripting and programmatic use.
-   **Line Numbering**: Prepend line numbers to every line with the `-n` or `--numbers` flag.
-   **Flexible Input**:
    -   Process multiple files and directories.
    -   Read from `stdin`, allowing it to be used in shell pipelines.
    -   Handle NUL-terminated file lists from commands like `find ... -print0` using the `-0` or `--null` flag.
-   **Path Manipulation**: Use `--strip N` to remove leading path components from file headers, cleaning up output for nested projects.
-   **Robust and Fast**: Built with Rust for performance and memory safety.

## Installation

### From source with Cargo

If you have the Rust toolchain installed, you can build and install `rucat` directly from source. From the root of the project repository:

```bash
cargo install --path .
```

The binary will be placed in `~/.cargo/bin`.

## Usage

### Basic Usage

```bash
# Display a single file
rucat src/main.rs

# Display multiple files
rucat README.md Cargo.toml

# Pipe content from another command
ls -1 src/formatters | rucat
```

### Formatting Options

`rucat` defaults to the `markdown` format. Use the `-f` or `--format` flag to change it.

```bash
# Use the ANSI formatter with a width of 80 columns and line numbers
rucat -f ansi --ansi-width 80 -n src/main.rs

# Use the simple ASCII format
rucat -f ascii src/main.rs

# Get JSON output for scripting
rucat -f json src/main.rs > output.json
```

### Advanced Input

`rucat` can process a NUL-separated list of files from standard input, which is safer and more robust than using `xargs`. This is especially useful with `find`.

```bash
# Find all Rust files and display them using the markdown format
find src -name "*.rs" -print0 | rucat -0 -f markdown
```

### Path Stripping

When working with deep directory structures, the full file path can be noisy. Use `--strip` to shorten the paths in the output headers.

```bash
# Before stripping: === src/formatters/ansi.rs ===
rucat -f ascii src/formatters/ansi.rs

# After stripping 2 components: === ansi.rs ===
rucat -f ascii --strip 2 src/formatters/ansi.rs
```

## Contributing

Contributions are welcome! If you have a feature request, bug report, or pull request, please feel free to open an issue or submit a PR.

## License

This project is licensed under the GNU General Public License v3.0.

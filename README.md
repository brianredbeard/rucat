# `rucat` - A versatile `cat` clone

![rucat image](/.assets/rucat.png)

`rucat` is a modern, feature-rich replacement for the classic `cat` utility,
written in Rust. It goes beyond simple file concatenation, offering multiple
output formats, line numbering, and syntax-aware formatting, making it an ideal
tool for developers, system administrators, and anyone working with code or text
files in the terminal.

## Features

- **Multiple Output Formats**: Display content in various styles, including:
  - `ansi`: Nicely formatted with borders (width-configurable via
    `--ansi-width`).
  - `utf8`: Fancy UTF-8 box-drawing borders (width-configurable via
    `--ansi-width`).
  - `markdown`: GitHub-flavored Markdown code blocks, with automatic language
    detection from the file extension.
  - `ascii`: Simple `=== file.txt ===` headers for easy separation.
  - `xml`: Structured XML output with file and line metadata.
  - `json`: A clean JSON array of file entries, perfect for scripting and
    programmatic use.
  - `pretty`: Syntax highlighting for a wide range of languages. Syntax is
    chosen based on: 1) the `--pretty-syntax` flag, 2) a Vim modeline in the
    file (e.g., `vim: ft=rust`), or 3) the file extension.
- **Line Numbering**: Prepend line numbers to every line with the `-n` or
  `--numbers` flag.
- **Flexible Input**:
  - Process multiple files and directories.
  - Read from `stdin`, allowing it to be used in shell pipelines.
  - Handle NUL-terminated file lists from commands like `find ... -print0` using
    the `-0` or `--null` flag.
- **Path Manipulation**: Use `--strip N` to remove leading path components from
  file headers, cleaning up output for nested projects.
- **Packaging**:
  - Built-in support for generating `.deb` packages for Debian/Ubuntu systems
    via `cargo deb`.
  - Built-in support for generating `.rpm` packages for Fedora/RHEL/CentOS
    systems via `cargo rpm`.
- **Robust and Fast**: Built with Rust for performance and memory safety.

## Installation

### From source with Cargo

If you have the Rust toolchain installed, you can build and install `rucat`
directly from source. From the root of the project repository:

```bash
cargo install --path .
```

### Building Packages

This project is configured to build `.deb` and `.rpm` packages using standard
Cargo tooling. First, ensure you have the necessary packaging subcommands
installed:

```bash
cargo install cargo-deb cargo-rpm
```

Then, you can build the packages from the project root:

```bash
# Build .deb package (output in target/debian/)
cargo deb

# Build .rpm package (output in target/release/rpmbuild/RPMs/)
cargo rpm build
```

The binary will be placed in `~/.cargo/bin`.

### Cross-compiling from macOS to Linux

When cross-compiling from macOS to Linux, `rustc` needs a C-language toolchain
that can link executables for the Linux target. The native macOS toolchain
cannot do this. You can install the necessary toolchains using Homebrew, but it
requires adding a new formula tap first.

**This is a one-time setup for your development machine. No changes are needed
for the project's code.**

1. **Install Cross-Compilation Toolchains with Homebrew**

   First, tap the repository that contains the toolchains. Then, install them.

   ```bash
   brew tap messense/macos-cross-toolchains
   brew install aarch64-unknown-linux-gnu
   brew install x86_64-unknown-linux-gnu
   ```

1. **Configure Cargo to Use the New Linkers**

   You must tell Cargo to use these newly installed linkers for the respective
   targets. Create or edit the file `~/.cargo/config.toml` (this is Cargo's
   global configuration file in your home directory, not your project directory)
   and add the following content:

   ```toml
   [target.aarch64-unknown-linux-gnu]
   linker = "aarch64-unknown-linux-gnu-gcc"

   [target.x86_64-unknown-linux-gnu]
   linker = "x86_64-unknown-linux-gnu-gcc"
   ```

After completing these two steps, your system will be properly configured for
cross-compilation, and `make cross-build-all` should succeed.

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

`rucat` defaults to the `markdown` format. Use the `-f` or `--format` flag to
change it.

```bash
# Use the ANSI formatter with a width of 80 columns and line numbers
rucat -f ansi --ansi-width 80 -n src/main.rs

# Use the simple ASCII format
rucat -f ascii src/main.rs

# Get JSON output for scripting
rucat -f json src/main.rs > output.json

# Use the pretty-printer with syntax highlighting
rucat -f pretty src/main.rs

# Force a specific syntax for the pretty-printer
rucat -f pretty --pretty-syntax sh < 'my-script-without-extension'
```

### Advanced Input

`rucat` can process a NUL-separated list of files from standard input, which is
safer and more robust than using `xargs`. This is especially useful with `find`.

```bash
# Find all Rust files and display them using the markdown format
find src -name "*.rs" -print0 | rucat -0 -f markdown
```

### Path Stripping

When working with deep directory structures, the full file path can be noisy.
Use `--strip` to shorten the paths in the output headers.

```bash
# Before stripping: === src/formatters/ansi.rs ===
rucat -f ascii src/formatters/ansi.rs

# After stripping 2 components: === ansi.rs ===
rucat -f ascii --strip 2 src/formatters/ansi.rs
```

## Configuration

`rucat` can be configured with a TOML file to set your preferred default
options. Create a file at `~/.config/rucat/config.toml` (or the equivalent XDG
config path on your OS).

Command-line arguments will always override settings from this file.

**Example `config.toml`:**

```toml
# Default output format.
# Possible values: "ansi", "utf8", "markdown", "ascii", "xml", "json",
# "pretty"
format = "ansi"

# Default to showing line numbers.
numbers = true

# Default number of path components to strip from filenames.
strip = 1

# Default width for the "ansi" and "utf8" formatters.
ansi_width = 120
utf8_width = 120

# Default syntax for the "pretty" formatter.
pretty_syntax = "rust"
```

## Contributing

Contributions are welcome! If you have a feature request, bug report, or pull
request, please feel free to open an issue or submit a PR.

This tool was proudly co-written using [Aider](https://github.com/Aider-AI/aider)

## License

This project is licensed under the GNU General Public License v3.0.

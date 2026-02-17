# `rucat` - A versatile `cat` clone

![rucat image](/.assets/rucat.png)
_our mascot, "Rucus"_

`rucat` is `cat` reborn for the era of LLMs. A critical tool for every prompt
engineer.

Written in Rust. It goes beyond simple file concatenation, offering multiple
output formats, line numbering, syntax-aware formatting, and clipboard support,
making it an ideal tool for developers, system administrators, and anyone working
with code or text files in the terminal.

## Powering AI and Development Workflows

In modern development, AI assistants are invaluable for complex tasks like
debugging, refactoring, or repository analysis. However, their effectiveness
hinges on the quality and completeness of the context you provide. Manually
copying and pasting from numerous files is slow and error-prone.

`rucat` excels at rapidly consolidating context from multiple files into a
single, well-structured block of text, perfect for an AI prompt.

If you're troubleshooting a complex `git` history issue and need to provide an
AI with the full state of your repository's refs and logs, you can run:

```bash
# Quickly gather all relevant git state files into one block
$ rucat .git/{HEAD,config,info/exclude,logs/HEAD,logs/refs/heads/*}

# Or copy directly to clipboard for pasting into AI chat
$ rucat --copy .git/{HEAD,config,info/exclude,logs/HEAD,logs/refs/heads/*}
```

````
```
---
File: .git/HEAD
---
```
ref: refs/heads/main
```
---
File: .git/config
---
```
[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
    ignorecase = true
    precomposeunicode = true
[remote "origin"]
    url = https://github.com/poundifdef/SmoothMQ
    fetch = +refs/heads/*:refs/remotes/origin/*
[branch "main"]
    remote = origin
    merge = refs/heads/main
```
---
File: .git/info/exclude
---
```
# git ls-files --others --exclude-from=.git/info/exclude
# Lines that start with '#' are comments.
# For a project mostly in C, the following would be a good set of
# exclude patterns (uncomment them if you want to use them):
# *.[oa]
# *~
```
---
File: .git/logs/HEAD
---
```
0000000000000000000000000000000000000000 2770c4b21a3755e95652c24b71ac7cad87b532dc Brian 'redbeard' Harrington <redbeard@dead-city.org> 1718820643 -0700	clone: from https://github.com/poundifdef/SmoothMQ
```
---
File: .git/logs/refs/heads/main
---
```
0000000000000000000000000000000000000000 2770c4b21a3755e95652c24b71ac7cad87b532dc Brian 'redbeard' Harrington <redbeard@dead-city.org> 1718820643 -0700	clone: from https://github.com/poundifdef/SmoothMQ
```
````

This command instantly generates a clean, markdown-formatted output, with each
file's content neatly separated and labeled:

The resulting text can be piped to your clipboard and pasted directly into your
AI chat, delivering complete and unambiguous context in seconds. This makes
`rucat` an essential tool for accelerating AI-driven development.

## Features

- **Multiple Output Formats**: Display content in various styles, including:
  - `ansi`: Nicely formatted with borders (width-configurable via
    `--ansi-width`).
  - `utf8`: Fancy UTF-8 box-drawing borders (width-configurable via
    `--utf8-width`).
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
- **Clipboard Support**: Copy output directly to the system clipboard with the
  `-c` or `--copy` flag. Supports multiple clipboard providers:
  - Native clipboard on Windows, macOS, and Linux (X11/Wayland)
  - Terminal escape sequences (OSC 52 for tmux/SSH, OSC 5522 for Kitty)
  - Automatic provider detection based on your environment
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
- **Robust and Fast**: Built with Rust for performance and memory safety.

## Installation

### From Homebrew (for macOS users)

If you are on macOS and have Homebrew installed, you can install `rucat` with a single command:

```bash
brew install brianredbeard/rucat/rucat
```

### From source with Cargo

If you have the Rust toolchain installed, you can build and install `rucat`
directly from source. From the root of the project repository:

```bash
cargo install --path .
```

To build without clipboard support (reduces dependencies):

```bash
cargo install --path . --no-default-features
```

### Building Packages

This project is configured to build `.deb` packages using standard Cargo
tooling. First, ensure you have the necessary packaging subcommand installed:

```bash
cargo install cargo-deb
```

Then, you can build the package from the project root:

```bash
# Build .deb package (output in target/debian/)
cargo deb
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

## Command-Line Options

`rucat` offers a rich set of command-line flags to customize its behavior. Flags can be placed before or after file arguments.

### Main Options

-   `FILES...`
    -   **Description**: One or more files or directories to process. If a directory is provided, `rucat` will recursively process all files within it. If no files are provided, `rucat` reads from standard input.
    -   **Example**: `rucat README.md src/`

### Formatting Options

These flags control the appearance of the output.

-   `-f, --format <FORMAT>`
    -   **Description**: Sets the output format.
    -   **Values**:
        -   `ansi`: Formats output with ANSI box-drawing characters.
        -   `utf8`: Formats output with fancy UTF-8 box-drawing characters.
        -   `markdown`: (Default) Wraps each file's content in a GitHub-flavored Markdown code block.
        -   `ascii`: Separates files with a simple `=== file.txt ===` header.
        -   `xml`: Produces structured XML output.
        -   `json`: Produces a JSON array of file entries, ideal for scripting.
        -   `pretty`: Applies syntax highlighting based on file type.
    -   **Example**: `rucat -f pretty src/main.rs`

-   `-n, --numbers`
    -   **Description**: Adds a gutter with line numbers to the output.
    -   **Example**: `rucat -n Cargo.toml`

-   `--ansi-width <WIDTH>`
    -   **Description**: Sets the minimum interior width for the `ansi` formatter.
    -   **Default**: `80`
    -   **Example**: `rucat -f ansi --ansi-width 120 src/lib.rs`

-   `--utf8-width <WIDTH>`
    -   **Description**: Sets the minimum interior width for the `utf8` formatter.
    -   **Default**: `80`
    -   **Example**: `rucat -f utf8 --utf8-width 100 src/lib.rs`

-   `--pretty-syntax <SYNTAX>`
    -   **Description**: Overrides syntax detection for the `pretty` formatter, forcing a specific language.
    -   **Example**: `echo "echo 'Hello'" | rucat -f pretty --pretty-syntax sh`

### Metadata Options

These flags are used to display file metadata. They are most effective with formats like `ascii`, `ansi`, or `utf8` that display headers.

-   `--stat`
    -   **Description**: Includes detailed file metadata (size, permissions, timestamps, etc.) in the output header for each file.
    -   **Example**: `rucat --stat /etc/hosts`

-   `--show-binary-xattrs`
    -   **Description**: By default, binary extended attributes are hidden when using `--stat`. This flag forces them to be displayed.
    -   **Example**: `rucat --stat --show-binary-xattrs some_file_with_binary_xattrs`

-   `--binary-format <FORMAT>`
    -   **Description**: Sets the display format for binary extended attributes when `--show-binary-xattrs` is used.
    -   **Values**:
        -   `hexdump`: (Default) An `xxd`-style hexadecimal and ASCII view.
        -   `base64`: Standard Base64 encoding.
        -   `intel-hex`: The Intel HEX object file format.
        -   `raw-hex`: A continuous string of hexadecimal characters with no spaces or headers.
    -   **Example**: `rucat --stat --show-binary-xattrs --binary-format base64 ...`

-   `--hex-width <BYTES>`
    -   **Description**: Sets the number of bytes per line for the `hexdump` binary format.
    -   **Default**: `16`
    -   **Example**: `rucat --stat --show-binary-xattrs --hex-width 32 ...`

-   `--base64-width <CHARS>`
    -   **Description**: Sets the maximum number of characters per line for the `base64` binary format.
    -   **Default**: `76`
    -   **Example**: `rucat --stat --show-binary-xattrs --binary-format base64 --base64-width 100 ...`

### Input/Output Options

These flags control how `rucat` reads input and handles output.

-   `-c, --copy`
    -   **Description**: Copies the entire output to the system clipboard while still printing it to standard output. Requires the `clipboard` feature.
    -   **Example**: `rucat -c src/main.rs`

-   `-0, --null`
    -   **Description**: Reads a NUL-separated list of file paths from standard input. This is useful for safely handling filenames that contain whitespace or special characters, especially when used with `find -print0`.
    -   **Example**: `find . -name "*.rs" -print0 | rcat -0`

-   `--strip <N>`
    -   **Description**: Removes `N` leading path components from filenames in the output headers.
    -   **Example**: `rucat --strip 2 src/formatters/ansi.rs` will display the header for `ansi.rs`.

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

This tool was proudly co-written using
[Aider](https://github.com/Aider-AI/aider)

## License

This project is licensed under the GNU General Public License v3.0.

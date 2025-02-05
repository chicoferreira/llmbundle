# llmbundle

A lightweight command-line utility that uses glob patterns to quickly search and retrieve project files, outputting
their contents to the clipboard.
Use this to easily paste your project files into LLMs.

## Installation

Ensure you have [Rust](https://rust-lang.org) installed. Then clone the repository and install the project:

```sh
git clone https://github.com/chicoferreira/llmbundle
cd llmbundle
cargo install --path .
```

This will install the `llmbundle` binary globally on your system.

## Usage

```sh
llmbundle [OPTIONS] [patterns]...
```

### Options

- `<patterns>`: One or more glob patterns to match files. Patterns without a directory separator are treated as matching
  any file with that name (e.g., `main.rs` is normalized to `**/main.rs`).
- `--root <root>`: Root directory for file search (default: `.`).
- `--max-depth <max_depth>`: Maximum directory depth to traverse.
- `--output <OUTPUT>`: Output destination; either `stdout` or `clipboard` (default: clipboard).
- `-v, --verbose`: Enable verbose logging.

### Examples

Search for all files in the current directory (files in `.gitignore` are ignored):

```shell
llmbundle
```

```shell
Files matched
+ Cargo.toml
+ LICENSE
+ Cargo.lock
+ README.md
+ src/main.rs

Copied 5 files to clipboard totalling 1395 lines, 3105 words and 37596 characters.
```

Search for all Rust files in the current directory and its subdirectories:

```sh
llmbundle '*.rs'
```

Search for a file named `main.rs` anywhere in the directory tree:

```sh
llmbundle 'main.rs'
```

Search for files in the `src` folder that have the `.rs` extension:

```sh
llmbundle 'src/*.rs'
```

Exclude files by prefixing the pattern with `!`:

```sh
llmbundle '*.rs' '!test.rs'
```

## License

This project is licensed under the [MIT License](LICENSE).
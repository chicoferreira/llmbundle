use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::Path;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Glob patterns to match files (supports wildcards and tilde expansion)
    patterns: Vec<String>,

    /// Maximum directory depth to traverse (optional)
    #[arg(long, help = "Set the maximum depth for directory traversal")]
    max_depth: Option<usize>,

    /// Root directory to start the search from
    #[arg(long, default_value = ".", help = "Root directory for file search")]
    root: String,

    /// Output destination: stdout or clipboard
    #[arg(long, value_enum, default_value_t = Output::Clipboard, help = "Choose the output destination"
    )]
    output: Output,

    /// Enable verbose logging for debugging purposes
    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Output {
    Stdout,
    Clipboard,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let (pos_globs, neg_globs) = build_glob_sets(&args.patterns)?;

    if args.verbose {
        eprintln!("Searching in root: {}", args.root);
    }

    let walker = WalkBuilder::new(&args.root)
        .max_depth(args.max_depth)
        .build();

    let root = Path::new(&args.root);

    let mut matching_files = Vec::new();
    for entry in walker {
        let entry = entry?;
        let path = entry.path().strip_prefix(root).unwrap_or(entry.path());
        if !path.is_file() {
            continue;
        }
        if !pos_globs.is_match(path) || neg_globs.is_match(path) {
            continue;
        }
        if args.verbose {
            eprintln!("Matched file: {}", path.display());
        }
        matching_files.push(path.to_owned());
    }

    if args.verbose {
        eprintln!("Total matching files: {}", matching_files.len());
    }

    let file_outputs: Vec<String> = matching_files
        .par_iter()
        .map(|path| process_file(path, args.verbose))
        .collect();

    let output_buffer = file_outputs.join("\n");

    match args.output {
        Output::Stdout => println!("{}", output_buffer),
        Output::Clipboard => {
            print_summary(&matching_files, &output_buffer);
            Clipboard::new()
                .context("Failed to initialize clipboard")?
                .set_text(output_buffer)
                .context("Failed to set clipboard text")?;
            if args.verbose {
                eprintln!("Output copied to clipboard.");
            }
        }
    }

    Ok(())
}

const FORMAT: &str = r#"[file name]: {file_name}
[file content begin]
{file_content}
[file content end]
"#;

fn process_file(path: &Path, verbose: bool) -> String {
    if verbose {
        eprintln!("Reading file: {}", path.display());
    }

    let content = std::fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", path.display(), e);
            String::new()
        });

    FORMAT
        .replace("{file_name}", path.display().to_string().as_str())
        .replace("{file_content}", &content)
}

fn normalize_pattern(pattern: &str) -> String {
    if pattern.contains(std::path::MAIN_SEPARATOR) {
        pattern.to_string()
    } else {
        format!("**/{}", pattern)
    }
}

fn build_glob_sets(patterns: &[String]) -> Result<(GlobSet, GlobSet)> {
    let (mut pos, mut neg) = (GlobSetBuilder::new(), GlobSetBuilder::new());

    let mut pos_builder_is_empty = true;

    for pattern in patterns {
        let pattern = pattern.trim();

        let (pattern, builder) = {
            if pattern.starts_with('!') {
                let pattern = pattern.trim_start_matches('!');
                if pattern.is_empty() {
                    continue;
                }
                // If the pattern starts with a '!', add to negative patterns
                (pattern, &mut neg)
            } else {
                // Otherwise add to positive patterns
                pos_builder_is_empty = false;
                (pattern, &mut pos)
            }
        };

        let pattern = normalize_pattern(pattern);

        let expanded = shellexpand::full(&pattern)
            .with_context(|| format!("Failed to expand pattern: {}", pattern))?
            .into_owned();
        let glob = Glob::new(&expanded)
            .with_context(|| format!("Invalid glob pattern after expansion: {}", expanded))?;

        builder.add(glob);
    }

    if pos_builder_is_empty {
        pos.add(Glob::new("**").expect("** is valid pattern"));
    }

    let pos_set = pos.build().context("Failed to build positive glob set")?;
    let neg_set = neg.build().context("Failed to build negative glob set")?;

    Ok((pos_set, neg_set))
}

fn print_summary(matching_files: &[std::path::PathBuf], buffer: &str) {
    println!("{}", "Files matched".blue().bold());
    if matching_files.is_empty() {
        println!("{}", "No files matched.".red());
        return;
    }
    for file in matching_files {
        println!("{} {}", "+".red(), file.display());
    }

    println!(
        "\nCopied {} to clipboard totalling {}, {} and {}.",
        format!("{} files", matching_files.len()).bold(),
        format!("{} lines", buffer.lines().count()).bold(),
        format!("{} words", buffer.split_whitespace().count()).bold(),
        format!("{} characters", buffer.chars().count()).bold()
    );
}

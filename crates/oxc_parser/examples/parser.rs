#![expect(clippy::print_stdout)]
//! # Parser Example
//!
//! This example demonstrates how to use the Oxc parser to parse JavaScript and TypeScript files.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_parser --example parser [filename...] [--ast] [--estree] [--comments]
//! ```
//!
//! ## Options
//!
//! - `--ast`: Display the parsed AST structure
//! - `--estree`: Display the ESTree representation
//! - `--comments`: Display extracted comments
//! - `--repeat N`: Parse each file N times (useful for profiling)
//!
//! Accepts files or directories. Directories are walked recursively for
//! `.js`, `.mjs`, `.cjs`, `.jsx`, `.ts`, `.mts`, `.cts`, `.tsx` files.

use std::{fs, path::Path, time::Instant};

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `just watch "cargo run -p oxc_parser --example parser"`

const EXTENSIONS: &[&str] = &["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"];

fn collect_files(path: &Path) -> Vec<std::path::PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    if !path.is_dir() {
        return vec![];
    }
    let mut files = Vec::new();
    collect_files_recursive(path, &mut files);
    files.sort();
    files
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip node_modules and hidden directories
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with('.') && name != "node_modules" {
                collect_files_recursive(&path, files);
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if EXTENSIONS.contains(&ext) {
                files.push(path);
            }
        }
    }
}

fn parse_file(
    path: &Path,
    source_text: &str,
    source_type: SourceType,
    show_ast: bool,
    show_estree: bool,
    show_comments: bool,
) -> bool {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions { parse_regular_expression: true, ..ParseOptions::default() })
        .parse();
    let mut program = ret.program;

    if show_comments {
        println!("Comments ({}):", path.display());
        for comment in &program.comments {
            let s = comment.content_span().source_text(source_text);
            println!("{s}");
        }
    }

    if show_ast {
        println!("AST ({}):", path.display());
        println!("{program:#?}");
    }

    if show_estree {
        Utf8ToUtf16::new(source_text).convert_program(&mut program);
        if source_type.is_javascript() {
            println!("ESTree AST ({}):", path.display());
            println!("{}", program.to_pretty_estree_js_json(false));
        } else {
            println!("TS-ESTree AST ({}):", path.display());
            println!("{}", program.to_pretty_estree_ts_json(false));
        }
    }

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.to_string());
            println!("{error:?}");
        }
        return false;
    }
    true
}

/// Parse and display information about JavaScript or TypeScript files
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    let show_ast = args.contains("--ast");
    let show_estree = args.contains("--estree");
    let show_comments = args.contains("--comments");
    let repeat: usize =
        args.opt_value_from_str("--repeat").map_err(|e| e.to_string())?.unwrap_or(1);

    // Collect all free arguments as file/directory paths
    let mut paths: Vec<String> = Vec::new();
    loop {
        match args.free_from_str::<String>() {
            Ok(p) => paths.push(p),
            Err(_) => break,
        }
    }
    if paths.is_empty() {
        paths.push("test.js".to_string());
    }

    // Expand directories into individual files
    let mut files = Vec::new();
    for p in &paths {
        let path = Path::new(p);
        let found = collect_files(path);
        if found.is_empty() {
            return Err(format!("No parseable files found at '{p}'"));
        }
        files.extend(found);
    }

    // Pre-read all files
    let sources: Vec<(std::path::PathBuf, String, SourceType)> = files
        .iter()
        .filter_map(|path| {
            let source_text = fs::read_to_string(path)
                .map_err(|e| eprintln!("Skipping {}: {e}", path.display()))
                .ok()?;
            let source_type = SourceType::from_path(path).ok()?;
            Some((path.clone(), source_text, source_type))
        })
        .collect();

    let total_bytes: usize = sources.iter().map(|(_, src, _)| src.len()).sum();
    let total_lines: usize = sources.iter().map(|(_, src, _)| src.lines().count()).sum();

    println!(
        "Parsing {} file(s), {total_lines} lines, {:.2} MB, {repeat}x...",
        sources.len(),
        total_bytes as f64 / 1_000_000.0
    );

    let start = Instant::now();
    let mut success = 0usize;
    let mut failure = 0usize;

    for iteration in 0..repeat {
        for (path, source_text, source_type) in &sources {
            let ok =
                parse_file(path, source_text, *source_type, show_ast, show_estree, show_comments);
            // Only count on last iteration to avoid double-counting
            if iteration == repeat - 1 {
                if ok {
                    success += 1;
                } else {
                    failure += 1;
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let total_parses = sources.len() * repeat;

    if failure == 0 {
        println!("Parsed Successfully.");
    } else {
        println!("{failure} file(s) had errors.");
    }

    println!(
        "{total_parses} parse(s) in {:.3}s ({:.2} MB/s, {:.0} files/s)",
        elapsed.as_secs_f64(),
        (total_bytes * repeat) as f64 / 1_000_000.0 / elapsed.as_secs_f64(),
        total_parses as f64 / elapsed.as_secs_f64(),
    );

    Ok(())
}

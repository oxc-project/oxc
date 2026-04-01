#![expect(clippy::print_stdout)]
//! # Checker Example
//!
//! Run the type checker on TypeScript files and report diagnostics + timing.
//!
//! ## Usage
//!
//! Single file:
//! ```bash
//! cargo run -p oxc_checker --example checker -- test.ts
//! ```
//!
//! Multiple files:
//! ```bash
//! cargo run -p oxc_checker --example checker -- src/a.ts src/b.ts src/c.ts
//! ```
//!
//! With timing (for benchmarking against tsgo):
//! ```bash
//! cargo run -p oxc_checker --example checker --release -- --timing src/*.ts
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    let show_timing = args
        .iter()
        .position(|a| a == "--timing")
        .map(|i| {
            args.remove(i);
            i
        })
        .is_some();

    if args.is_empty() {
        eprintln!("Usage: checker [--timing] <file.ts> [file2.ts ...]");
        return ExitCode::FAILURE;
    }

    let file_paths: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();

    // Canonicalize paths
    let file_paths: Vec<PathBuf> = file_paths
        .into_iter()
        .filter_map(|p| match p.canonicalize() {
            Ok(canonical) => Some(canonical),
            Err(e) => {
                eprintln!("Error: cannot read {}: {e}", p.display());
                None
            }
        })
        .collect();

    if file_paths.is_empty() {
        return ExitCode::FAILURE;
    }

    let arena = oxc_types::TypeArena::with_capacity(1024);

    if file_paths.len() == 1 {
        // Single-file mode: use Project for global types, Checker for the file
        let source = std::fs::read_to_string(&file_paths[0]).unwrap();
        let source_type = oxc_span::SourceType::from_path(&file_paths[0]).unwrap_or_default();
        let project = oxc_project::Project::new(&arena);

        let parse_start = std::time::Instant::now();
        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, source_type).parse();
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        let bind_start = std::time::Instant::now();
        let semantic = oxc_semantic::SemanticBuilder::new().build(&parsed.program).semantic;
        let bind_ms = bind_start.elapsed().as_secs_f64() * 1000.0;

        let check_start = std::time::Instant::now();
        let mut checker = oxc_checker::Checker::new_with_host(
            &semantic,
            &arena,
            &project,
            file_paths[0].to_string_lossy().to_string(),
            1,
        );
        checker.check_program(&parsed.program);
        let check_ms = check_start.elapsed().as_secs_f64() * 1000.0;

        let diagnostics = checker.take_diagnostics();
        let total_ms = parse_ms + bind_ms + check_ms;

        for diag in &diagnostics {
            eprintln!("{diag}");
        }

        if show_timing {
            println!();
            println!("Files:    1");
            println!("Parse:    {parse_ms:.1}ms");
            println!("Bind:     {bind_ms:.1}ms");
            println!("Check:    {check_ms:.1}ms");
            println!("Total:    {total_ms:.1}ms");
            println!("Errors:   {}", diagnostics.len());
            println!("Types:    {}", arena.len());
        } else {
            println!("{} error(s)", diagnostics.len());
        }
    } else {
        // Multi-file mode: use Project
        let mut project = oxc_project::Project::new_multi(&arena, file_paths);
        let result = project.check_all();

        let total_errors: usize = result.diagnostics.iter().map(|(_, d)| d.len()).sum();
        for (path, diags) in &result.diagnostics {
            for diag in diags {
                eprintln!("{}:{diag}", path.display());
            }
        }

        if show_timing {
            println!();
            println!("Files:    {}", result.files_checked);
            println!("Parse:    {:.1}ms", result.timing.parse_ms);
            println!("Bind:     {:.1}ms", result.timing.bind_ms);
            println!("Check:    {:.1}ms", result.timing.check_ms);
            println!("Total:    {:.1}ms", result.timing.total_ms);
            println!("Errors:   {total_errors}");
            println!("Types:    {}", arena.len());
        } else {
            println!("{total_errors} error(s) across {} files", result.files_checked);
        }
    }

    ExitCode::SUCCESS
}

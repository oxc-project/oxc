#![expect(clippy::print_stdout)]
//! # Checker Example
//!
//! Check a TypeScript project (isolated declarations forced).
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p oxc_checker --example checker -- path/to/project-or-tsconfig.json
//! ```

use std::{env, path::Path, sync::Arc};

use oxc_checker::check_project;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let result = match check_project(Path::new(&arg)) {
        Ok(result) => result,
        Err(err) => {
            println!("{err}");
            std::process::exit(2);
        }
    };

    // Colored unicode output, like oxlint.
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode());
    let cwd = env::current_dir().ok();

    let mut error_count = 0usize;
    let mut file_count = 0usize;
    for file in &result.files {
        if file.diagnostics.is_empty() {
            continue;
        }
        file_count += 1;
        error_count += file.diagnostics.len();
        // Paths print relative to the working directory, like tsc/oxlint.
        let display_path = cwd
            .as_ref()
            .and_then(|cwd| file.path.strip_prefix(cwd).ok())
            .unwrap_or(&file.path);
        let source =
            Arc::new(NamedSource::new(display_path.to_string_lossy(), file.source_text.clone()));
        let mut rendered = String::new();
        for diagnostic in &file.diagnostics {
            let report = diagnostic.clone().with_source_code(Arc::clone(&source));
            let _ = handler.render_report(&mut rendered, report.as_ref());
            rendered.push('\n');
        }
        print!("{rendered}");
    }

    let summary = format!(
        "Checked {} files: found {error_count} error(s) in {file_count} file(s).",
        result.files.len()
    );
    if error_count > 0 {
        println!("\x1b[1;31m{summary}\x1b[0m");
        std::process::exit(1);
    }
    println!("\x1b[1;32m{summary}\x1b[0m");
}

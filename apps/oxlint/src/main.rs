use std::io::BufWriter;

use oxlint::cli::{CliRunResult, CliRunner, init_miette, init_tracing, lint_command, run_lsp};

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = lint_command().run();

    // Both LSP and CLI use `tracing` for logging
    init_tracing();

    // If --lsp flag is set, run the language server
    if command.lsp {
        run_lsp(None).await;
        return CliRunResult::LintSucceeded;
    }

    init_miette();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    // Run without external linter (no JS plugins)
    CliRunner::new(command, None).run(&mut stdout)
}

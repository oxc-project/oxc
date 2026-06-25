use std::io::BufWriter;

use oxlint::{
    cli::{CliRunResult, CliRunner, init_miette, init_tracing, lint_command},
    lsp::run_lsp,
};

fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = lint_command().run();

    // Both LSP and CLI use `tracing` for logging
    init_tracing();

    // If --lsp flag is set, run the language server.
    //
    // The language server is the only path that needs an async runtime, so the Tokio runtime is
    // created here rather than via `#[tokio::main]`. The CLI lint path is fully synchronous
    // (Rayon-based); `#[tokio::main]` would otherwise spawn one idle Tokio worker thread per core
    // on every lint invocation.
    if command.lsp {
        let runtime =
            tokio::runtime::Runtime::new().expect("Failed to build the Tokio runtime for the LSP");
        return runtime.block_on(async {
            run_lsp(
                None,
                #[cfg(feature = "napi")]
                None,
            )
            .await;
            CliRunResult::LintSucceeded
        });
    }

    init_miette();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    // Run without external linter (no JS plugins)
    CliRunner::new(command, None).run(&mut stdout)
}

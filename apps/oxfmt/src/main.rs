use std::io::BufWriter;

use oxfmt::{CliRunResult, FormatRunner, format_command, init_miette, init_tracing, run_lsp};

// Pure Rust CLI entry point.
// For JS CLI entry point, see `run.rs` exported by `lib.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    // Handle LSP mode
    if command.misc_options.lsp {
        run_lsp().await;
        return CliRunResult::None;
    }

    // Otherwise, CLI mode
    init_tracing();
    init_miette();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut stderr = BufWriter::new(std::io::stderr());
    FormatRunner::new(command).run(&mut stdout, &mut stderr)
}

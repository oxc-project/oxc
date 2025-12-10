use std::io::BufWriter;

use oxfmt::cli::{CliRunResult, FormatRunner, format_command, init_miette, init_tracing};
use oxfmt::lsp::run_lsp;
use oxfmt::stdin::StdinRunner;

// Pure Rust CLI entry point.
// For JS CLI entry point, see `format()` exported by `main_napi.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    // Handle LSP mode
    if command.misc_options.lsp {
        run_lsp().await;
        return CliRunResult::None;
    }

    // Handle Stdin mode
    if let Some(stdin_filepath) = command.misc_options.stdin_filepath {
        init_tracing();
        init_miette();

        let mut stdin = std::io::stdin();
        let mut stdout = BufWriter::new(std::io::stdout());
        let mut stderr = BufWriter::new(std::io::stderr());
        return StdinRunner::new(stdin_filepath, command.basic_options.config).run(
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
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

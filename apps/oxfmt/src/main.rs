use oxfmt::cli::{
    CliRunResult, FormatRunner, format_command, init_miette, init_rayon, init_tracing,
};
use oxfmt::init::run_init;
use oxfmt::lsp::run_lsp;

// Pure Rust CLI entry point.
// For JS CLI entry point, see `format()` exported by `main_napi.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    // Handle --init mode
    if command.misc_options.init {
        return run_init();
    }

    // Handle LSP mode
    if command.misc_options.lsp {
        run_lsp().await;
        return CliRunResult::None;
    }

    // Otherwise, CLI mode
    init_tracing();
    init_miette();
    init_rayon(command.misc_options.threads);

    FormatRunner::new(command).run()
}

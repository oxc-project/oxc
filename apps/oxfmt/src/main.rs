use oxfmt::cli::{
    CliRunResult, FormatRunner, Mode, format_command, init_miette, init_rayon, init_tracing,
};
use oxfmt::init::run_init;
use oxfmt::lsp::run_lsp;

// Pure Rust CLI entry point.
// For JS CLI entry point, see `format()` exported by `main_napi.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    match command.mode {
        Mode::Init => run_init(),
        Mode::Lsp => {
            run_lsp().await;
            CliRunResult::None
        }
        Mode::Cli(_) => {
            init_tracing();
            init_miette();
            init_rayon(command.runtime_options.threads);
            FormatRunner::new(command).run()
        }
    }
}

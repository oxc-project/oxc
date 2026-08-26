use oxfmt::cli::{CliRunResult, FormatCommand, WalkRunner, init_rayon, init_tracing};

// Pure Rust CLI entry point.
// This CLI only supports the basic `Cli` mode.
// For full featured JS CLI entry point, see `run_cli()` exported by `main_napi.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = FormatCommand::parse();

    init_tracing();
    init_rayon(command.runtime_options.threads);
    WalkRunner::new(command).run()
}

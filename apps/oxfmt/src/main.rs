use oxfmt::cli::{
    CliRunResult, FormatRunner, format_command, init_miette, init_rayon, init_tracing,
};

// Pure Rust CLI entry point.
// This CLI only supports the basic `Cli` mode.
// For full featured JS CLI entry point, see `run_cli()` exported by `main_napi.rs`.

#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    init_tracing();
    init_miette();
    init_rayon(command.runtime_options.threads);
    FormatRunner::new(command).run()
}

#[cfg(not(target_family = "wasm"))]
use oxfmt::cli::{CliRunResult, WalkRunner, format_command, init_miette, init_rayon, init_tracing};

// The bin target is not built for wasm (only the cdylib is; building both would
// collide on the `oxfmt.wasm` output name and ship a non-reactor module).
// This stub only keeps `cargo check --target wasm32-*` green.
#[cfg(target_family = "wasm")]
fn main() {}

// Pure Rust CLI entry point.
// This CLI only supports the basic `Cli` mode.
// For full featured JS CLI entry point, see `run_cli()` exported by `main_napi.rs`.

// On wasm, tokio only supports the current-thread runtime
#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() -> CliRunResult {
    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    init_tracing();
    init_miette();
    init_rayon(command.runtime_options.threads);
    WalkRunner::new(command).run()
}

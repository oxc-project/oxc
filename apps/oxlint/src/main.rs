// NB: Miri does not support custom allocators
#[cfg(all(feature = "allocator", not(miri), not(target_env = "msvc"), not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(all(feature = "allocator", not(miri), target_os = "windows"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxlint::cli::{CliRunResult, LintRunner, Runner};
use std::io::BufWriter;

fn main() -> CliRunResult {
    init_tracing();
    init_miette();

    let command = oxlint::cli::lint_command().run();
    command.handle_threads();
    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    LintRunner::new(command).run(&mut stdout)
}

// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("OXC_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

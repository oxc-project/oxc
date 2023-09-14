#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliRunResult, LintRunner, Runner};

fn main() -> CliRunResult {
    init_tracing();

    let command = oxc_cli::lint_command().fallback_to_usage().run();
    command.handle_threads();
    LintRunner::new(command.lint_options).run()
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
    registry().with(fmt::layer()).with(EnvFilter::from_env("OXC_LOG")).init();
}

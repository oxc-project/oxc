#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliRunResult, FormatRunner, Runner};

fn main() -> CliRunResult {
    oxc_diagnostics::init_miette();

    let command = oxc_cli::format_command().fallback_to_usage().run();
    command.handle_threads();
    FormatRunner::new(command.format_options).run()
}

#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliCommand, CliRunResult, FormatRunner, LintRunner, Runner};

fn main() -> CliRunResult {
    let options = oxc_cli::cli_command().fallback_to_usage().run();
    options.handle_threads();
    match options {
        CliCommand::Lint(options) => LintRunner::new(options).run(),
        CliCommand::Format(options) => FormatRunner::new(options).run(),
    }
}

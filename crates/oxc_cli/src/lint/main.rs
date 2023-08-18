#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliRunResult, LintRunner, Runner};

fn main() -> CliRunResult {
    let command = oxc_cli::lint_command().fallback_to_usage().run();
    command.handle_threads();
    LintRunner::new(command.lint_options).run()
}

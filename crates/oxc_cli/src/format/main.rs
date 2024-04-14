#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(debug_assertions))]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(not(debug_assertions))]
#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliRunResult, FormatRunner, Runner};

fn main() -> CliRunResult {
    init_miette();

    let command = oxc_cli::format_command().fallback_to_usage().run();
    command.handle_threads();
    FormatRunner::new(command.format_options).run()
}

// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

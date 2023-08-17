#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{command, CliRunResult, LintRunner, Runner, TypeCheckRunner};

fn main() -> CliRunResult {
    let matches = command().get_matches();

    if let Some(threads) = matches.get_one::<usize>("threads") {
        rayon::ThreadPoolBuilder::new().num_threads(*threads).build_global().unwrap();
    }

    let Some((subcommand, matches)) = matches.subcommand() else { return CliRunResult::None };

    match subcommand {
        LintRunner::NAME => LintRunner::new(matches).run(),
        TypeCheckRunner::NAME => TypeCheckRunner::new(matches).run(),
        _ => CliRunResult::None,
    }
}

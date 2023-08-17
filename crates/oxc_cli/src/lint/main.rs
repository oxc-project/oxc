#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Arg, Command};
use oxc_cli::{CliRunResult, LintRunner, Runner};

pub fn command() -> Command {
    LintRunner::command()
        .bin_name("oxlint")
        .version("alpha")
        .author("Boshen")
        .about("Linter for the JavaScript Oxidation Compiler")
        .arg_required_else_help(true)
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_parser(clap::value_parser!(usize))
                .help("Number of threads to use. Set to 1 for using only 1 CPU core."),
        )
}

fn main() -> CliRunResult {
    let matches = command().get_matches();

    if let Some(threads) = matches.get_one::<usize>("threads") {
        rayon::ThreadPoolBuilder::new().num_threads(*threads).build_global().unwrap();
    }

    LintRunner::new(&matches).run()
}

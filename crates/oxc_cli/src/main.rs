#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{
    command, matches_to_lint_options, CliRunResult, LintRunner, TypeCheckOptions, TypeCheckRunner,
};

fn main() -> CliRunResult {
    let matches = command().get_matches();

    if let Some(threads) = matches.get_one::<usize>("threads") {
        rayon::ThreadPoolBuilder::new().num_threads(*threads).build_global().unwrap();
    }

    let Some((subcommand, matches)) = matches.subcommand() else { return CliRunResult::None };

    match subcommand {
        "lint" => {
            let options = matches_to_lint_options(matches);

            if options.list_rules {
                LintRunner::print_rules();
                return CliRunResult::None;
            }

            LintRunner::run(options)
        }
        "check" => {
            let options = TypeCheckOptions::from(matches);
            TypeCheckRunner::new(options).run()
        }
        _ => CliRunResult::None,
    }
}

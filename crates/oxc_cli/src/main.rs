#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::env;

use oxc_cli::{
    command, CliRunResult, LintOptions, LintRunner, LintRunnerWithModuleTree, TypeCheckOptions,
    TypeCheckRunner,
};

fn main() -> CliRunResult {
    let matches = command().get_matches();

    if let Some(threads) = matches.get_one::<usize>("threads") {
        rayon::ThreadPoolBuilder::new().num_threads(*threads).build_global().unwrap();
    }

    let Some((subcommand, matches)) = matches.subcommand() else { return CliRunResult::None };

    match subcommand {
        "lint" => {
            let options = LintOptions::from(matches);

            if options.list_rules {
                LintRunner::print_rules();
                return CliRunResult::None;
            }

            if matches!(env::var("OXC_MODULE_TREE"), Ok(x) if x == "true" || x == "1") {
                LintRunnerWithModuleTree::new(options).run()
            } else {
                LintRunner::new(options).run()
            }
        }
        "check" => {
            let options = TypeCheckOptions::from(matches);
            TypeCheckRunner::new(options).run()
        }
        _ => CliRunResult::None,
    }
}

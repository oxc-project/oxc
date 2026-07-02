use std::num::NonZeroUsize;

use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

use oxc_coverage::{AppArgs, TestData};

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let app_args = AppArgs {
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    // Init rayon thread pool
    let thread_count = if app_args.debug {
        1
    } else {
        std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
    };
    // Match Linux's default main-thread stack (8 MB) so deeply-nested expressions
    // (e.g. typescript/tests/cases/compiler/binderBinaryExpressionStressJs.ts)
    // don't overflow macOS's 2 MB worker default.
    ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .stack_size(8 * 1024 * 1024)
        .build_global()
        .unwrap();

    // Load test data lazily: `runtime` and `transpiler` load their own inputs,
    // so they must not require every fixture suite to be checked out
    // (`run_all` also loads internally).
    let load = || TestData::load(app_args.filter.as_deref());

    let task = command.as_deref().unwrap_or("default");
    match task {
        "parser" => app_args.run_parser(&load()),
        "semantic" => app_args.run_semantic(&load()),
        "codegen" => app_args.run_codegen(&load()),
        "formatter" => app_args.run_formatter(&load()),
        "transformer" => app_args.run_transformer(&load()),
        "transpiler" => app_args.run_transpiler(),
        "minifier" => app_args.run_minifier(&load()),
        "runtime" => app_args.run_runtime(),
        "estree" => app_args.run_estree(&load()),
        "estree_tokens" => app_args.run_estree_tokens(&load()),
        "types" => app_args.run_types(&load()),
        "all" => app_args.run_all_with(&load()),
        _ => app_args.run_all(),
    }
}

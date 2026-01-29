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
        std::thread::available_parallelism().map(NonZeroUsize::get).unwrap_or(1)
    };
    ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();

    // Load all test data once
    let data = TestData::load(app_args.filter.as_deref());

    let task = command.as_deref().unwrap_or("default");
    match task {
        "parser" => app_args.run_parser(&data),
        "semantic" => app_args.run_semantic(&data),
        "codegen" => app_args.run_codegen(&data),
        "formatter" => app_args.run_formatter(&data),
        "transformer" => app_args.run_transformer(&data),
        "transpiler" => app_args.run_transpiler(),
        "minifier" => app_args.run_minifier(&data),
        "runtime" => app_args.run_runtime(),
        "estree" => app_args.run_estree(&data),
        "all" => {
            app_args.run_all();
            app_args.run_runtime();
        }
        _ => app_args.run_all(),
    }
}

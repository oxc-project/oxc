use std::num::NonZeroUsize;

use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

use oxc_coverage::AppArgs;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let args = AppArgs {
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    // Init rayon thread pool
    let thread_count = if args.debug {
        1
    } else {
        std::thread::available_parallelism().map(NonZeroUsize::get).unwrap_or(1)
    };
    ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();

    let task = command.as_deref().unwrap_or("default");
    match task {
        "parser" => args.run_parser(),
        "semantic" => args.run_semantic(),
        "codegen" => args.run_codegen(),
        // "formatter" => args.run_formatter(),
        "transformer" => args.run_transformer(),
        "transpiler" => args.run_transpiler(),
        "minifier" => args.run_minifier(),
        "runtime" => args.run_runtime(),
        "estree" => args.run_estree(),
        "all" => {
            args.run_default();
            args.run_runtime();
        }
        _ => args.run_default(),
    }
}

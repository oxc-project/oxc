use std::num::NonZeroUsize;

use oxc_allocator::AllocatorPool;
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

    // Initialize allocator pool with the same thread count
    let allocator_pool = AllocatorPool::new(thread_count);

    let task = command.as_deref().unwrap_or("default");
    match task {
        "parser" => args.run_parser(&allocator_pool),
        "semantic" => args.run_semantic(&allocator_pool),
        "codegen" => args.run_codegen(&allocator_pool),
        "formatter" => args.run_formatter(&allocator_pool),
        "transformer" => args.run_transformer(&allocator_pool),
        "transpiler" => args.run_transpiler(&allocator_pool),
        "minifier" => args.run_minifier(&allocator_pool),
        "runtime" => args.run_runtime(&allocator_pool),
        "estree" => args.run_estree(&allocator_pool),
        "all" => {
            args.run_default(&allocator_pool);
            args.run_runtime(&allocator_pool);
        }
        _ => args.run_default(&allocator_pool),
    }
}

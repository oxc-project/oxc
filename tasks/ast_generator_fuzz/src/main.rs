use oxc_ast_generator_fuzz::{default_thread_count, run_range_with_progress};

fn main() -> Result<(), pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();
    let seed = args.opt_value_from_str("--seed")?.unwrap_or(0_u64);
    let iterations = args.opt_value_from_str("--iterations")?.unwrap_or(1_000_u64);
    let threads = args.opt_value_from_str("--threads")?.unwrap_or_else(default_thread_count);

    run_range_with_progress(seed, iterations, threads);

    Ok(())
}

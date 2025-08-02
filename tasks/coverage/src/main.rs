use oxc_coverage::AppArgs;
use oxc_tasks_common::{CommonCliOptions, configure_thread_pool, get_subcommand};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let command = get_subcommand(&mut args);
    let cli_options = CommonCliOptions::from_args(&mut args);

    let args = AppArgs {
        debug: cli_options.debug,
        filter: cli_options.filter,
        detail: cli_options.detail,
        diff: cli_options.diff,
    };

    configure_thread_pool(args.debug);

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

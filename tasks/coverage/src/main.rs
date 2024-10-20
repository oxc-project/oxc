use oxc_coverage::AppArgs;
use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let args = AppArgs {
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    if args.debug {
        ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
    }

    let task = command.as_deref().unwrap_or("default");
    match task {
        "parser" => args.run_parser(),
        "semantic" => args.run_semantic(),
        "codegen" => args.run_codegen(),
        "codegen-runtime" => args.run_codegen_runtime(),
        // "prettier" => args.run_prettier(),
        "transformer" => args.run_transformer(),
        "transpiler" => args.run_transpiler(),
        "minifier" => args.run_minifier(),
        "all" => {
            args.run_all();
            args.run_codegen_runtime();
        }
        _ => args.run_all(),
    };
}

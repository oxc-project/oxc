use oxc_coverage::AppArgs;
use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let args = AppArgs {
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    let task = command.as_deref().unwrap_or("default");

    match task {
        "parser" => args.run_parser(),
        "codegen" => args.run_codegen(),
        "codegen-runtime" => args.run_codegen_runtime(),
        "prettier" => args.run_prettier(),
        "transformer" => args.run_transformer(),
        "minifier" => args.run_minifier(),
        "linter" => args.run_linter(),
        "v8_test262_status" => args.run_sync_v8_test262_status(),
        _ => args.run_all(),
    };
}

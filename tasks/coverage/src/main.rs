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
        "js" | "test262" => args.run_test262(),
        "babel" => args.run_babel(),
        "ts" | "typescript" => args.run_typescript(),
        "formatter" => args.run_formatter(),
        "minifier" => args.run_minifier(),
        _ => args.run_all(),
    };
}

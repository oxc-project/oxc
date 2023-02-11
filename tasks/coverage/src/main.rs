use oxc_coverage::{AppArgs, Suite, Test262Case, Test262Suite};
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

    let run_test262 = || {
        Test262Suite::<Test262Case>::new().run("Test262", &args);
    };

    match task {
        "js" | "test262" => run_test262(),
        _ => {
            run_test262();
        }
    };
}

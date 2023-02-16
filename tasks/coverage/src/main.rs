use oxc_coverage::{
    AppArgs, BabelCase, BabelSuite, PrinterTest262Case, Suite, Test262Case, Test262Suite,
    TypeScriptCase, TypeScriptSuite,
};
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

    let run_babel = || {
        BabelSuite::<BabelCase>::new().run("Babel", &args);
    };

    let run_typescript = || {
        TypeScriptSuite::<TypeScriptCase>::new().run("TypeScript", &args);
    };

    let run_printer = || {
        Test262Suite::<PrinterTest262Case>::new().run("Printer", &args);
    };

    match task {
        "js" | "test262" => run_test262(),
        "babel" => run_babel(),
        "ts" | "typescript" => run_typescript(),
        "printer" => run_printer(),
        _ => {
            run_test262();
            run_babel();
            run_typescript();
            run_printer();
        }
    };
}

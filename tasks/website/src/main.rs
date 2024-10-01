#![allow(clippy::print_stdout, clippy::print_stderr)]
use pico_args::Arguments;
use website::linter;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let task = command.as_deref().unwrap_or("default");

    match task {
        "linter-schema-json" => linter::print_schema_json(),
        "linter-schema-markdown" => linter::print_schema_markdown(),
        "linter-cli" => linter::print_cli(),
        "linter-rules" => linter::print_rules(args),
        _ => println!("Missing task command."),
    }
}

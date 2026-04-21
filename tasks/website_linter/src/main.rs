#![expect(clippy::print_stderr)]

use pico_args::Arguments;

mod cli;
mod json_schema;
mod rules;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");
    let task = command.as_deref().unwrap_or("default");

    match task {
        "schema-json" => json_schema::print_schema_json(),
        "schema-markdown" => json_schema::print_schema_markdown(),
        "cli" => cli::print_cli(),
        "rules" => rules::print_rules(args),
        _ => eprintln!("Missing task command."),
    }
}

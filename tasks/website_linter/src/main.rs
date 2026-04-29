#![expect(clippy::print_stderr)]

use pico_args::Arguments;

mod built_in_rules;
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
        "built-in-rules-ts" => built_in_rules::print_built_in_rules_ts(),
        "cli" => cli::print_cli(),
        "rules" => rules::print_rules(args),
        _ => eprintln!("Missing task command."),
    }
}

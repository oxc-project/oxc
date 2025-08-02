#![expect(clippy::print_stderr)]

use oxc_tasks_common::get_subcommand;
use website::linter;

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let command = get_subcommand(&mut args);

    let task = command.as_deref().unwrap_or("default");

    match task {
        "linter-schema-json" => linter::print_schema_json(),
        "linter-schema-markdown" => linter::print_schema_markdown(),
        "linter-cli" => linter::print_cli(),
        "linter-rules" => linter::print_rules(args),
        _ => eprintln!("Missing task command."),
    }
}

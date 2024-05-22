use pico_args::Arguments;

use website::linter;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let task = command.as_deref().unwrap_or("default");

    match task {
        "linter-schema-json" => linter::generate_schema_json(),
        "linter-schema-markdown" => linter::generate_schema_markdown(),
        "linter-cli" => linter::generate_cli(),
        "linter-rules" => linter::generate_rules(),
        _ => println!("Missing task command."),
    }
}

#![expect(clippy::print_stderr)]

use pico_args::Arguments;

mod cli;
mod json_schema;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");
    let task = command.as_deref().unwrap_or("default");

    match task {
        "cli" => cli::print_cli(),
        "schema-json" => {
            let root = project_root::get_project_root().expect("project root");
            json_schema::write_schema_json(
                root.join("npm/oxfmt/configuration_schema.json").to_str().expect("valid path"),
            );
        }
        "schema-markdown" => json_schema::print_schema_markdown(),
        _ => eprintln!("Missing task command."),
    }
}

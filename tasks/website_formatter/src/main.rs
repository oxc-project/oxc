#![expect(clippy::print_stderr)]

use pico_args::Arguments;

mod cli;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");
    let task = command.as_deref().unwrap_or("default");

    match task {
        "cli" => cli::print_cli(),
        _ => eprintln!("Missing task command."),
    }
}

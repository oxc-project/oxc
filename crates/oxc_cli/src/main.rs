#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{Cli, CliOptions, CliRunResult, Command};

fn main() -> CliRunResult {
    if let Some(command) = Command::new().build().get_matches().subcommand() {
        let (subcommand, matches) = command;
        let cli_options = CliOptions::try_from(matches);
        if let Ok(cli_options) = cli_options {
            let cli = Cli::new(cli_options);

            if subcommand == "lint" {
                return cli.lint();
            }
            return CliRunResult::None;
        }
    }
    CliRunResult::None
}

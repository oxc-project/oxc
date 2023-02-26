#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;

use oxc_cli::{Cli, CliOptions, CliRunResult, Command};

fn main() -> CliRunResult {
    if let Some(command) = Command::new().build().get_matches().subcommand() {
        let (subcommand, matches) = command;
        let cli_options = CliOptions::try_from(matches);

        if let Ok(cli_options) = cli_options {
            let cli = Cli::new(cli_options);

            if subcommand == "lint" {}
        } else {
            return CliRunResult::PathNotFound { path: cli_options.unwrap().path };
        }

        // let cli = Cli::new(cli_options);

        // if cli.cli_options.path.canonicalize().is_err() {
        //     CliRunResult::PathNotFound { path: cli.cli_options.path };
        // }
    }

    // match Command::new().build().get_matches().subcommand() {
    //     Some(("lint", matches)) => {
    //         let path = matches.get_one::<PathBuf>("path").unwrap();

    //         if path.canonicalize().is_err() {
    //             return CliRunResult::PathNotFound { path: path.clone() };
    //         }

    //         let quiet = matches.get_one::<String>("quiet").is_some();
    //         Cli::lint(path, CliOptions { quiet })
    //     }
    //     _ => CliRunResult::None,
    // }
}

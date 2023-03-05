#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{Cli, CliOptions, CliRunResult, Command};
use oxc_diagnostics::miette;

fn main() -> CliRunResult {
    // convert source code with hard tabs into 4 spaces,
    // otherwise error label spans will have a wrong offset.
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().tab_width(4).build())))
        .unwrap();

    if let Some(command) = Command::new().build().get_matches().subcommand() {
        let (subcommand, matches) = command;
        let cli_options = CliOptions::try_from(matches);
        if let Ok(cli_options) = cli_options {
            // if cli_options.fix {
            //   Git::new().verify()?;
            // }

            let cli = Cli::new(cli_options);

            if subcommand == "lint" {
                return cli.lint();
            }
            return CliRunResult::None;
        }
    }
    CliRunResult::None
}

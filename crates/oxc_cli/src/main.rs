#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{Cli, CliOptions, CliRunResult, Command, Subcommand};
use oxc_diagnostics::miette;

fn main() -> CliRunResult {
    // convert source code with hard tabs into 4 spaces,
    // otherwise error label spans will have a wrong offset.
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().tab_width(4).build())))
        .unwrap();

    let top_level_cmd: Command = argh::from_env();

    let Subcommand::Lint(command) = top_level_cmd.inner;
    let cli_options = CliOptions::try_from(&command);
    if let Ok(cli_options) = cli_options {
        // if cli_options.fix {
        //   Git::new().verify()?;
        // }

        let cli = Cli::new(cli_options);

        return cli.lint();
    }
    CliRunResult::None
}

#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{command, CliRunResult, LintOptions, LintRunner};
use oxc_diagnostics::miette;

fn main() -> CliRunResult {
    // convert source code with hard tabs into 4 spaces,
    // otherwise error label spans will have a wrong offset.
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().tab_width(4).build())))
        .unwrap();

    let matches = command().get_matches();
    let Some((subcommand, matches)) = matches.subcommand() else {
        return CliRunResult::None
    };

    match subcommand {
        "lint" => {
            let options = LintOptions::from(matches);
            LintRunner::new(options).run()
        }
        _ => CliRunResult::None,
    }
}

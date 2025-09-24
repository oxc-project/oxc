mod command;
mod format;
mod reporter;
mod result;
mod service;
mod walk;

pub mod cli {
    pub use crate::{
        command::{FormatCommand, format_command},
        format::FormatRunner,
        result::CliRunResult,
    };
}

use std::io::BufWriter;

use cli::{CliRunResult, FormatRunner, format_command};

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

pub fn format() -> CliRunResult {
    init_tracing();
    init_miette();

    let mut args = std::env::args_os();
    // If first arg is `node`, also skip script path (`node script.js ...`).
    // Otherwise, just skip first arg (`oxfmt ...`).
    if args.next().is_some_and(|arg| arg == "node") {
        args.next();
    }
    let args = args.collect::<Vec<_>>();

    // Parse command line arguments
    let command = match format_command().run_inner(&*args) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            return if e.exit_code() == 0 {
                // e.g. `-V` and `--help`
                CliRunResult::None
            } else {
                // e.g. Unknown options
                CliRunResult::InvalidOptionConfig
            };
        }
    };
    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    FormatRunner::new(command).run(&mut stdout)
}

/// To debug `oxc_formatter`:
/// `OXC_LOG=oxc_formatter oxfmt`
fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("OXC_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

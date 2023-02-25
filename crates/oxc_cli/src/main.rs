#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;

use oxc_cli::{Cli, CliRunResult, Command};

fn main() -> CliRunResult {
    match Command::new().build().get_matches().subcommand() {
        Some(("lint", matches)) => {
            let mut paths = vec![];

            for path in matches.get_many::<PathBuf>("path").unwrap() {
                let globbed = glob::glob(&path.to_string_lossy())
                    .expect("Failed to read glob pattern")
                    .map(|path| path.expect("Failed to read glob pattern"))
                    .collect::<Vec<_>>();

                if globbed.is_empty() && path.canonicalize().is_err() {
                    return CliRunResult::PathNotFound { path: path.clone() };
                }

                paths.extend(globbed);
            }

            Cli::lint(&paths)
        }
        _ => CliRunResult::None,
    }
}

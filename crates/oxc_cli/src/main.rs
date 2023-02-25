#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;
use std::process::{ExitCode, Termination};

use oxc_cli::{Cli, Command, LintResult};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    PathNotFound { path: PathBuf },
    LintResult(Vec<LintResult>),
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::from(0),
            Self::PathNotFound { path } => {
                println!("Path {} does not exist.", path.to_string_lossy());
                ExitCode::from(1)
            }
            Self::LintResult(results) => {
                println!("Checked {} files.", results.len());

                let has_errors = results.iter().any(|result| !result.diagnostics.is_empty());

                if has_errors {
                    for LintResult { path, diagnostics } in results {
                        println!("File: {path:?}");
                        for diagnostic in diagnostics {
                            println!("{diagnostic:?}");
                        }
                    }
                    return ExitCode::from(1);
                }
                ExitCode::from(0)
            }
        }
    }
}

fn main() -> CliRunResult {
    match Command::new().build().get_matches().subcommand() {
        Some(("lint", matches)) => {
            let path = matches.get_one::<PathBuf>("path").unwrap();

            if path.canonicalize().is_err() {
                return CliRunResult::PathNotFound { path: path.clone() };
            }

            let result = Cli::lint(path);
            CliRunResult::LintResult(result.unwrap())
        }
        _ => CliRunResult::None,
    }
}

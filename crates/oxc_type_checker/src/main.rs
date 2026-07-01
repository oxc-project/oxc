//! `oxc_tsc` — experimental CLI that discovers a `tsconfig.json` and prints the project's
//! TypeScript files (the root files from the config's `files`/`include`/`exclude` specs).
//!
//! ```bash
//! oxc_tsc                       # discover tsconfig.json from the current directory upward
//! oxc_tsc -p path/to/tsconfig.json
//! oxc_tsc -p path/to/dir        # uses <dir>/tsconfig.json
//! ```
#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

use bpaf::Bpaf;
use oxc_type_checker::project;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Discover a tsconfig.json and list the project's files.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
struct DiscoverCommand {
    /// Path to a `tsconfig.json` file, or a directory containing one
    #[bpaf(short('p'), long("project"), argument("PATH"))]
    project: Option<PathBuf>,
}

enum RunResult {
    Success,
    Failed,
}

impl Termination for RunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::Failed => ExitCode::FAILURE,
        }
    }
}

fn main() -> RunResult {
    let command = discover_command().run();

    match project::discover(command.project.as_deref()) {
        Ok(result) => {
            for diagnostic in &result.diagnostics {
                eprintln!("{diagnostic:?}");
            }
            // tsgo's internal order is literal -> wildcard -> json; sort for stable CLI output.
            let mut files = result.files;
            files.sort();
            files.dedup();
            for file in &files {
                println!("{file}");
            }
            RunResult::Success
        }
        Err(error) => {
            eprintln!("error: {error}");
            RunResult::Failed
        }
    }
}

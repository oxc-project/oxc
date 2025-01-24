use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    InvalidOptions { message: String },
    PathNotFound { paths: Vec<PathBuf> },
    LintResult(LintResult),
    PrintConfigResult,
    ConfigFileInitResult { message: String },
}

/// A summary of a complete linter run.
#[derive(Debug, Default)]
pub struct LintResult {
    /// The number of files that were linted.
    pub number_of_files: usize,
    /// The number of warnings that were found.
    pub number_of_warnings: usize,
    /// The number of errors that were found.
    pub number_of_errors: usize,
    /// The exit unix code for, in general 0 or 1 (from `--deny-warnings` or `--max-warnings` for example)
    pub exit_code: ExitCode,
}

impl Termination for CliRunResult {
    #[allow(clippy::print_stdout, clippy::print_stderr)]
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::PrintConfigResult => ExitCode::from(0),
            Self::InvalidOptions { message } => {
                println!("Invalid Options: {message}");
                ExitCode::from(1)
            }
            Self::PathNotFound { paths } => {
                println!("Path {paths:?} does not exist.");
                ExitCode::from(1)
            }
            Self::LintResult(LintResult {
                number_of_files: _,    // ToDo: only for tests, make snapshots
                number_of_warnings: _, // ToDo: only for tests, make snapshots
                number_of_errors: _,
                exit_code,
            }) => exit_code,
            Self::ConfigFileInitResult { message } => {
                println!("{message}");
                ExitCode::from(0)
            }
        }
    }
}

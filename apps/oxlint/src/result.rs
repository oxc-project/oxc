use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    InvalidOptions {
        message: String,
    },
    PathNotFound {
        paths: Vec<PathBuf>,
    },
    /// The exit unix code for, in general 0 or 1 (from `--deny-warnings` or `--max-warnings` for example)
    LintResult(ExitCode),
    PrintConfigResult,
    ConfigFileInitResult {
        message: String,
    },
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
            Self::LintResult(exit_code) => exit_code,
            Self::ConfigFileInitResult { message } => {
                println!("{message}");
                ExitCode::from(0)
            }
        }
    }
}

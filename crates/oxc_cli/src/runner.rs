use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

/// A trait for exposing functionality to the CLI.
pub trait Runner: Send + Sync {
    type Options;

    fn new(matches: Self::Options) -> Self;

    /// Executes the runner, providing some result to the CLI.
    fn run(&self) -> CliRunResult;
}

#[derive(Debug)]
pub enum CliRunResult {
    None,
    IOError(crate::lint::Error),
    PathNotFound {
        paths: Vec<PathBuf>,
    },
    LintResult {
        duration: std::time::Duration,
        number_of_rules: usize,
        number_of_files: usize,
        number_of_warnings: usize,
        number_of_errors: usize,
        max_warnings_exceeded: bool,
    },
    TypeCheckResult {
        duration: std::time::Duration,
        number_of_diagnostics: usize,
    },
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::from(0),
            Self::PathNotFound { paths } => {
                println!("Path {paths:?} does not exist.");
                ExitCode::from(1)
            }
            Self::IOError(e) => {
                println!("IO Error: {e}");
                ExitCode::from(1)
            }
            Self::LintResult {
                duration,
                number_of_rules,
                number_of_files,
                number_of_warnings,
                number_of_errors,
                max_warnings_exceeded,
            } => {
                let ms = duration.as_millis();
                let threads = rayon::current_num_threads();
                let number_of_diagnostics = number_of_warnings + number_of_errors;

                if number_of_diagnostics > 0 {
                    println!();
                }

                println!(
                    "Finished in {ms}ms on {number_of_files} files with {number_of_rules} rules using {threads} threads."
                );

                if max_warnings_exceeded {
                    println!("Exceeded maximum number of warnings. Found {number_of_warnings}.");
                    return ExitCode::from(1);
                }

                if number_of_diagnostics > 0 {
                    let warnings = if number_of_warnings == 1 { "warning" } else { "warnings" };
                    let errors = if number_of_errors == 1 { "error" } else { "errors" };
                    println!(
                        "Found {number_of_warnings} {warnings} and {number_of_errors} {errors}."
                    );
                    return ExitCode::from(1);
                }

                // eslint does not print anything after success, so we do the same.
                // It is also standard to not print anything after success in the *nix world.
                ExitCode::from(0)
            }
            Self::TypeCheckResult { duration, number_of_diagnostics } => {
                let ms = duration.as_millis();
                println!("Finished in {ms}ms.");

                if number_of_diagnostics > 0 {
                    println!("Found {number_of_diagnostics} errors.");
                    return ExitCode::from(1);
                }

                ExitCode::from(0)
            }
        }
    }
}

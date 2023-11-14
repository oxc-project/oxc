use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
    time::Duration,
};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    InvalidOptions { message: String },
    PathNotFound { paths: Vec<PathBuf> },
    LintResult(LintResult),
    FormatResult(FormatResult),
    TypeCheckResult { duration: Duration, number_of_diagnostics: usize },
}

#[derive(Debug)]
pub struct LintResult {
    pub duration: Duration,
    pub number_of_rules: usize,
    pub number_of_files: usize,
    pub number_of_warnings: usize,
    pub number_of_errors: usize,
    pub max_warnings_exceeded: bool,
}

#[derive(Debug)]
pub struct FormatResult {
    pub duration: Duration,
    pub number_of_files: usize,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::from(0),
            Self::InvalidOptions { message } => {
                println!("Invalid Options: {message}");
                ExitCode::from(1)
            }
            Self::PathNotFound { paths } => {
                println!("Path {paths:?} does not exist.");
                ExitCode::from(1)
            }
            Self::LintResult(LintResult {
                duration,
                number_of_rules,
                number_of_files,
                number_of_warnings,
                number_of_errors,
                max_warnings_exceeded,
            }) => {
                let threads = rayon::current_num_threads();
                let number_of_diagnostics = number_of_warnings + number_of_errors;

                if number_of_diagnostics > 0 {
                    println!();
                }

                let time = Self::get_execution_time(&duration);
                let s = if number_of_files == 1 { "" } else { "s" };
                println!(
                    "Finished in {time} on {number_of_files} file{s} with {number_of_rules} rules using {threads} threads."
                );

                if max_warnings_exceeded {
                    println!("Exceeded maximum number of warnings. Found {number_of_warnings}.");
                    return ExitCode::from(1);
                }

                println!(
                    "Found {number_of_warnings} warning{} and {number_of_errors} error{}.",
                    if number_of_warnings == 1 { "" } else { "s" },
                    if number_of_errors == 1 { "" } else { "s" }
                );

                let exit_code = u8::from(number_of_errors > 0);
                ExitCode::from(exit_code)
            }
            Self::FormatResult(FormatResult { duration, number_of_files }) => {
                let threads = rayon::current_num_threads();
                let time = Self::get_execution_time(&duration);
                let s = if number_of_files == 1 { "" } else { "s" };
                println!(
                    "Finished in {time} on {number_of_files} file{s} using {threads} threads."
                );
                ExitCode::from(0)
            }
            Self::TypeCheckResult { duration, number_of_diagnostics } => {
                let time = Self::get_execution_time(&duration);
                println!("Finished in {time}.");

                if number_of_diagnostics > 0 {
                    println!("Found {number_of_diagnostics} errors.");
                    return ExitCode::from(1);
                }

                ExitCode::from(0)
            }
        }
    }
}

impl CliRunResult {
    fn get_execution_time(duration: &Duration) -> String {
        let ms = duration.as_millis();
        if ms < 1000 {
            format!("{ms}ms")
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        }
    }
}

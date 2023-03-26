use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    PathNotFound {
        paths: Vec<PathBuf>,
    },
    LintResult {
        duration: std::time::Duration,
        number_of_rules: usize,
        number_of_files: usize,
        number_of_warnings: usize,
        number_of_diagnostics: usize,
        max_warnings_exceeded: bool,
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
            Self::LintResult {
                duration,
                number_of_rules,
                number_of_files,
                number_of_warnings,
                number_of_diagnostics,
                max_warnings_exceeded,
            } => {
                let ms = duration.as_millis();
                let cpus = num_cpus::get();
                println!(
                    "Finished in {ms}ms on {number_of_files} files with {number_of_rules} rules using {cpus} cores."
                );

                if max_warnings_exceeded {
                    println!("Exceeded maximum number of warnings. Found {number_of_warnings}.");
                    return ExitCode::from(1);
                }

                if number_of_diagnostics > 0 {
                    println!("Found {number_of_diagnostics} errors.");
                    return ExitCode::from(1);
                }

                println!("Found no errors.");
                ExitCode::from(0)
            }
        }
    }
}

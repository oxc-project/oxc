use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    PathNotFound { path: PathBuf },
    LintResult { number_of_files: usize, number_of_diagnostics: usize },
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::from(0),
            Self::PathNotFound { path } => {
                println!("Path {} does not exist.", path.to_string_lossy());
                ExitCode::from(1)
            }
            Self::LintResult { number_of_files, number_of_diagnostics } => {
                println!("Checked {number_of_files} files.");

                if number_of_files > 0 {
                    println!("Found {number_of_diagnostics} diagnostics.");
                    return ExitCode::from(1);
                }

                ExitCode::from(0)
            }
        }
    }
}

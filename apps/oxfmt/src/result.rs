use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    LintNoFilesFound,
    // Failure
    InvalidOptionConfig,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::LintNoFilesFound => ExitCode::SUCCESS,
            Self::InvalidOptionConfig => ExitCode::FAILURE,
        }
    }
}

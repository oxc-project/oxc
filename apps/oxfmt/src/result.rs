use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    // Failure
    FormatNoFilesFound,
    InvalidOptionConfig,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded => ExitCode::SUCCESS,
            Self::FormatNoFilesFound | Self::InvalidOptionConfig => ExitCode::FAILURE,
        }
    }
}

use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    // Failure
    FormatNoFilesFound,
    InvalidOptionConfig,
    FormatMismatch,
    FormatFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded => ExitCode::SUCCESS,
            Self::FormatNoFilesFound
            | Self::InvalidOptionConfig
            | Self::FormatMismatch
            | Self::FormatFailed => ExitCode::FAILURE,
        }
    }
}

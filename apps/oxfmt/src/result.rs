use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    // Warning error
    InvalidOptionConfig,
    FormatMismatch,
    // Fatal error
    NoFilesFound,
    FormatFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded => ExitCode::from(0),
            Self::InvalidOptionConfig | Self::FormatMismatch => ExitCode::from(1),
            Self::NoFilesFound | Self::FormatFailed => ExitCode::from(2),
        }
    }
}

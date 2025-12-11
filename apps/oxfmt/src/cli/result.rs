use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    InitSucceeded,
    // Warning error
    InvalidOptionConfig,
    FormatMismatch,
    InitAborted,
    // Fatal error
    NoFilesFound,
    FormatFailed,
    InitFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded | Self::InitSucceeded => ExitCode::from(0),
            Self::InvalidOptionConfig | Self::FormatMismatch | Self::InitAborted => {
                ExitCode::from(1)
            }
            Self::NoFilesFound | Self::FormatFailed | Self::InitFailed => ExitCode::from(2),
        }
    }
}

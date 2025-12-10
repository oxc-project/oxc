use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    ConfigFileInitSucceeded,
    // Warning error
    InvalidOptionConfig,
    FormatMismatch,
    // Fatal error
    NoFilesFound,
    FormatFailed,
    ConfigFileInitFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded | Self::ConfigFileInitSucceeded => ExitCode::from(0),
            Self::InvalidOptionConfig | Self::FormatMismatch | Self::ConfigFileInitFailed => {
                ExitCode::from(1)
            }
            Self::NoFilesFound | Self::FormatFailed => ExitCode::from(2),
        }
    }
}

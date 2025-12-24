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

impl CliRunResult {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::None | Self::FormatSucceeded => 0,
            Self::InvalidOptionConfig | Self::FormatMismatch => 1,
            Self::NoFilesFound | Self::FormatFailed => 2,
        }
    }
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        ExitCode::from(self.exit_code())
    }
}

use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    // Success
    None,
    FormatSucceeded,
    InitSucceeded,
    MigrateSucceeded,
    // Warning error
    InvalidOptionConfig,
    FormatMismatch,
    InitAborted,
    MigrateAborted,
    // Fatal error
    NoFilesFound,
    FormatFailed,
    InitFailed,
    MigrateFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None | Self::FormatSucceeded | Self::InitSucceeded | Self::MigrateSucceeded => {
                ExitCode::from(0)
            }
            Self::InvalidOptionConfig
            | Self::FormatMismatch
            | Self::InitAborted
            | Self::MigrateAborted => ExitCode::from(1),
            Self::NoFilesFound | Self::FormatFailed | Self::InitFailed | Self::MigrateFailed => {
                ExitCode::from(2)
            }
        }
    }
}

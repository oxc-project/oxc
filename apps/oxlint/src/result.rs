use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    JsPluginWorkspaceSetupFailed,
    InvalidOptionConfig,
    InvalidOptionTsConfig,
    InvalidOptionTypeCheckWithoutTypeAware,
    InvalidOptionTypeCheckOnlyWithFix,
    InvalidOptionSeverityWithoutFilter,
    InvalidOptionSeverityWithoutPluginName,
    InvalidOptionSeverityWithoutRuleName,
    LintSucceeded,
    LintFoundErrors,
    LintUnprunedSuppressions,
    LintMaxWarningsExceeded,
    LintNoWarningsAllowed,
    LintNoFilesFound,
    PrintConfigResult,
    ConfigFileInitFailed,
    ConfigFileInitSucceeded,
    TsGoLintError,
    /// Failed to create or open the path passed to `--output-file`.
    OutputFileError,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None
            | Self::PrintConfigResult
            | Self::ConfigFileInitSucceeded
            | Self::LintSucceeded => ExitCode::SUCCESS,
            Self::ConfigFileInitFailed
            | Self::JsPluginWorkspaceSetupFailed
            | Self::LintFoundErrors
            | Self::LintNoFilesFound
            | Self::LintNoWarningsAllowed
            | Self::LintMaxWarningsExceeded
            | Self::InvalidOptionConfig
            | Self::InvalidOptionTsConfig
            | Self::InvalidOptionTypeCheckWithoutTypeAware
            | Self::InvalidOptionTypeCheckOnlyWithFix
            | Self::InvalidOptionSeverityWithoutFilter
            | Self::InvalidOptionSeverityWithoutPluginName
            | Self::InvalidOptionSeverityWithoutRuleName
            | Self::LintUnprunedSuppressions
            | Self::TsGoLintError => ExitCode::FAILURE,
            // Exit code 2 distinguishes `--output-file` I/O failures from lint errors (1).
            Self::OutputFileError => ExitCode::from(2),
        }
    }
}

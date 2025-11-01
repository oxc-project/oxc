use std::process::{ExitCode, Termination};

#[derive(Debug, PartialEq)]
pub enum CliRunResult {
    None,
    InvalidOptionConfig,
    InvalidOptionTsConfig,
    InvalidOptionSeverityWithoutFilter,
    InvalidOptionSeverityWithoutPluginName,
    InvalidOptionSeverityWithoutRuleName,
    LintSucceeded,
    LintFoundErrors,
    LintMaxWarningsExceeded,
    LintNoWarningsAllowed,
    LintNoFilesFound,
    PrintConfigResult,
    ConfigFileInitFailed,
    ConfigFileInitSucceeded,
    TsGoLintError,
    UnusedSuppressionsFound,
    SuppressionFileInvalid,
    SuppressionGenerationFailed,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None
            | Self::PrintConfigResult
            | Self::ConfigFileInitSucceeded
            | Self::LintSucceeded
            // ToDo: when oxc_linter (config) validates the configuration, we can use exit_code = 1 to fail
            | Self::LintNoFilesFound => ExitCode::SUCCESS,
            Self::ConfigFileInitFailed
            | Self::LintFoundErrors
            | Self::LintNoWarningsAllowed
            | Self::LintMaxWarningsExceeded
            | Self::InvalidOptionConfig
            | Self::InvalidOptionTsConfig
            | Self::InvalidOptionSeverityWithoutFilter
            | Self::InvalidOptionSeverityWithoutPluginName
            | Self::InvalidOptionSeverityWithoutRuleName
            | Self::TsGoLintError
            | Self::UnusedSuppressionsFound
            | Self::SuppressionFileInvalid
            | Self::SuppressionGenerationFailed => ExitCode::FAILURE,
        }
    }
}

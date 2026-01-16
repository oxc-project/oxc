use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub enum CliRunResult {
    None,
    InvalidOptionConfig,
    InvalidOptionTsConfig,
    InvalidOptionSeverityWithoutFilter,
    InvalidOptionSeverityWithoutPluginName,
    InvalidOptionSeverityWithoutRuleName,
    InvalidSuppressionOptions,
    LintSucceeded,
    LintFoundErrors,
    LintMaxWarningsExceeded,
    LintNoWarningsAllowed,
    LintNoFilesFound,
    LintUnusedSuppressions,
    PrintConfigResult,
    ConfigFileInitFailed,
    ConfigFileInitSucceeded,
    SuppressionsFileCreated,
    SuppressionsFilePruned,
    SuppressionsFileNotFound,
    SuppressionsFileError,
    TsGoLintError,
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None
            | Self::PrintConfigResult
            | Self::ConfigFileInitSucceeded
            | Self::LintSucceeded
            | Self::SuppressionsFileCreated
            | Self::SuppressionsFilePruned
            // ToDo: when oxc_linter (config) validates the configuration, we can use exit_code = 1 to fail
            | Self::LintNoFilesFound => ExitCode::SUCCESS,
            Self::ConfigFileInitFailed
            | Self::LintFoundErrors
            | Self::LintNoWarningsAllowed
            | Self::LintMaxWarningsExceeded
            | Self::LintUnusedSuppressions
            | Self::InvalidOptionConfig
            | Self::InvalidOptionTsConfig
            | Self::InvalidOptionSeverityWithoutFilter
            | Self::InvalidOptionSeverityWithoutPluginName
            | Self::InvalidOptionSeverityWithoutRuleName
            | Self::InvalidSuppressionOptions
            | Self::SuppressionsFileNotFound
            | Self::SuppressionsFileError
            | Self::TsGoLintError => ExitCode::FAILURE,
        }
    }
}

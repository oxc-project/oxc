mod checkstyle;
mod default;
mod github;
mod json;
mod junit;
mod stylish;
mod unix;
mod xml_utils;

use std::str::FromStr;
use std::time::Duration;

use checkstyle::CheckStyleOutputFormatter;
use github::GithubOutputFormatter;
use junit::JUnitOutputFormatter;
use stylish::StylishOutputFormatter;
use unix::UnixOutputFormatter;

use oxc_diagnostics::reporter::DiagnosticReporter;

use crate::output_formatter::{default::DefaultOutputFormatter, json::JsonOutputFormatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Default,
    /// GitHub Check Annotation
    /// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-a-notice-message>
    Github,
    Json,
    Unix,
    Checkstyle,
    Stylish,
    JUnit,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "default" => Ok(Self::Default),
            "unix" => Ok(Self::Unix),
            "checkstyle" => Ok(Self::Checkstyle),
            "github" => Ok(Self::Github),
            "stylish" => Ok(Self::Stylish),
            "junit" => Ok(Self::JUnit),
            _ => Err(format!("'{s}' is not a known format")),
        }
    }
}

/// Some extra lint information, which can be outputted
/// at the end of the command
pub struct LintCommandInfo {
    /// The number of files that were linted.
    pub number_of_files: usize,
    /// The number of lint rules that were run.
    pub number_of_rules: usize,
    /// The used CPU threads count
    pub threads_count: usize,
    /// Some reporters want to output the duration it took to finished the task
    pub start_time: Duration,
}

/// An Interface for the different output formats.
/// The Formatter is then managed by [`OutputFormatter`].
trait InternalFormatter {
    /// Print all available rules by oxlint
    fn all_rules(&self) -> Option<String> {
        None
    }

    /// At the end of the Lint command the Formatter can output extra information.
    fn lint_command_info(&self, _lint_command_info: &LintCommandInfo) -> Option<String> {
        None
    }

    /// oxlint words with [`DiagnosticService`](oxc_diagnostics::DiagnosticService),
    /// which uses a own reporter to output to stdout.
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter>;
}

pub struct OutputFormatter {
    internal: Box<dyn InternalFormatter>,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { internal: Self::get_internal_formatter(format) }
    }

    fn get_internal_formatter(format: OutputFormat) -> Box<dyn InternalFormatter> {
        match format {
            OutputFormat::Json => Box::<JsonOutputFormatter>::default(),
            OutputFormat::Checkstyle => Box::<CheckStyleOutputFormatter>::default(),
            OutputFormat::Github => Box::new(GithubOutputFormatter),
            OutputFormat::Unix => Box::<UnixOutputFormatter>::default(),
            OutputFormat::Default => Box::new(DefaultOutputFormatter),
            OutputFormat::Stylish => Box::<StylishOutputFormatter>::default(),
            OutputFormat::JUnit => Box::<JUnitOutputFormatter>::default(),
        }
    }

    /// Print all available rules by oxlint
    /// See [`InternalFormatter::all_rules`] for more details.
    pub fn all_rules(&self) -> Option<String> {
        self.internal.all_rules()
    }

    /// At the end of the Lint command we may output extra information.
    pub fn lint_command_info(&self, lint_command_info: &LintCommandInfo) -> Option<String> {
        self.internal.lint_command_info(lint_command_info)
    }

    /// Returns the [`DiagnosticReporter`] which then will be used by [`DiagnosticService`](oxc_diagnostics::DiagnosticService)
    /// See [`InternalFormatter::get_diagnostic_reporter`] for more details.
    pub fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        self.internal.get_diagnostic_reporter()
    }
}

#[cfg(test)]
mod test {
    use crate::tester::Tester;

    const TEST_CWD: &str = "fixtures/output_formatter_diagnostic";

    #[test]
    fn test_output_formatter_diagnostic_default() {
        let args = &["--format=default", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }

    /// disabled for windows
    /// json will output the offset which will be different for windows
    /// when there are multiple lines (`\r\n` vs `\n`)
    #[cfg(all(test, not(target_os = "windows")))]
    #[test]
    fn test_output_formatter_diagnostic_json() {
        let args = &["--format=json", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }

    #[test]
    fn test_output_formatter_diagnostic_checkstyle() {
        let args = &["--format=checkstyle", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }

    #[test]
    fn test_output_formatter_diagnostic_github() {
        let args = &["--format=github", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }

    /// disabled for windows
    /// stylish will output the offset which will be different for windows
    /// when there are multiple lines (`\r\n` vs `\n`)
    #[cfg(all(test, not(target_os = "windows")))]
    #[test]
    fn test_output_formatter_diagnostic_stylish() {
        let args = &["--format=stylish", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }

    #[test]
    fn test_output_formatter_diagnostic_junit() {
        let args = &["--format=junit", "test.js"];

        Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(args);
    }
}

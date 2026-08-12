mod agent;
mod checkstyle;
mod default;
mod github;
mod gitlab;
mod json;
mod junit;
mod sarif;
mod stylish;
mod unix;
mod xml_utils;

use std::{
    borrow::Cow,
    cell::RefCell,
    error::Error as StdError,
    fmt::{self, Debug, Display},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use oxc_diagnostics::{
    Diagnostic, Error, SourceCode,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{OxlintSuppressionFileAction, RuleTimingRecord};
use oxc_span::LabeledSpan;
use rustc_hash::FxHashSet;

use crate::output_formatter::{default::DefaultOutputFormatter, json::JsonOutputFormatter};

use agent::AgentOutputFormatter;
use checkstyle::CheckStyleOutputFormatter;
use github::GithubOutputFormatter;
use gitlab::GitlabOutputFormatter;
use junit::JUnitOutputFormatter;
use sarif::SarifOutputFormatter;
use stylish::StylishOutputFormatter;
use unix::UnixOutputFormatter;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Default,
    /// GitHub Check Annotation
    /// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-a-notice-message>
    Github,
    Gitlab,
    Json,
    Unix,
    Agent,
    Checkstyle,
    Stylish,
    JUnit,
    Sarif,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "default" => Ok(Self::Default),
            "unix" => Ok(Self::Unix),
            "agent" => Ok(Self::Agent),
            "checkstyle" => Ok(Self::Checkstyle),
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "stylish" => Ok(Self::Stylish),
            "junit" => Ok(Self::JUnit),
            "sarif" => Ok(Self::Sarif),
            _ => Err(format!("'{s}' is not a known format")),
        }
    }
}

/// Some extra lint information, which can be outputted
/// at the end of the command
pub struct LintCommandInfo {
    /// The number of files that were linted.
    pub number_of_files: usize,
    /// The number of lint rules that were run. If the number varies and can't be clearly
    /// computed, then this defaults to None.
    pub number_of_rules: Option<usize>,
    /// The used CPU threads count
    pub threads_count: usize,
    /// Some reporters want to output the duration it took to finished the task
    pub start_time: Duration,
    /// At least in default mode we want to notify if oxlint-suppressions.json was created or updated.
    pub oxlint_suppression_file_action: OxlintSuppressionFileAction,
    /// Optional per-rule timing records for debug timing output.
    pub rule_timings: Option<Vec<RuleTimingRecord>>,
}

impl LintCommandInfo {
    fn get_execution_time(start_time: &Duration) -> String {
        let ms = start_time.as_millis();
        if ms < 1000 { format!("{ms}ms") } else { format!("{:.1}s", start_time.as_secs_f64()) }
    }

    pub(super) fn format_execution_summary(&self) -> String {
        let time = Self::get_execution_time(&self.start_time);
        let s = if self.number_of_files == 1 { "" } else { "s" };

        let mut finished_text = if let Some(number_of_rules) = self.number_of_rules {
            format!(
                "Finished in {time} on {} file{s} with {} rules using {} threads.\n",
                self.number_of_files, number_of_rules, self.threads_count
            )
        } else {
            format!(
                "Finished in {time} on {} file{s} using {} threads.\n",
                self.number_of_files, self.threads_count
            )
        };

        let oxlint_suppression_action_text = match &self.oxlint_suppression_file_action {
            OxlintSuppressionFileAction::None
            | OxlintSuppressionFileAction::Exists
            | OxlintSuppressionFileAction::HasUnprunedSuppressions => String::new(),
            OxlintSuppressionFileAction::Created => {
                "Created 'oxlint-suppressions.json' in the root folder.\n".to_string()
            }
            OxlintSuppressionFileAction::Updated => {
                "Updated 'oxlint-suppressions.json'.\n".to_string()
            }
            OxlintSuppressionFileAction::Malformed(error)
            | OxlintSuppressionFileAction::UnableToPerformFsOperation(error) => {
                format!("{}\n", error.message)
            }
        };

        finished_text.insert_str(0, oxlint_suppression_action_text.as_ref());

        finished_text
    }
}

/// An Interface for the different output formats.
/// The Formatter is then managed by [`OutputFormatter`].
trait InternalFormatter {
    /// Print all available rules by oxlint
    fn all_rules(&self, _enabled_rules: FxHashSet<&str>) -> Option<String> {
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
    additional: Option<AdditionalOutputFormatter>,
}

struct AdditionalOutputFormatter {
    internal: Box<dyn InternalFormatter>,
    output: Rc<RefCell<String>>,
    silent_primary: bool,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { internal: Self::get_internal_formatter(format), additional: None }
    }

    pub fn new_with_additional_output(
        format: OutputFormat,
        additional_format: OutputFormat,
        silent_primary: bool,
    ) -> Self {
        Self {
            internal: Self::get_internal_formatter(format),
            additional: Some(AdditionalOutputFormatter {
                internal: Self::get_internal_formatter(additional_format),
                output: Rc::default(),
                silent_primary,
            }),
        }
    }

    fn get_internal_formatter(format: OutputFormat) -> Box<dyn InternalFormatter> {
        match format {
            OutputFormat::Json => Box::<JsonOutputFormatter>::default(),
            OutputFormat::Checkstyle => Box::<CheckStyleOutputFormatter>::default(),
            OutputFormat::Github => Box::new(GithubOutputFormatter),
            OutputFormat::Gitlab => Box::<GitlabOutputFormatter>::default(),
            OutputFormat::Unix => Box::<UnixOutputFormatter>::default(),
            OutputFormat::Agent => Box::<AgentOutputFormatter>::default(),
            OutputFormat::Default => Box::new(DefaultOutputFormatter),
            OutputFormat::Stylish => Box::<StylishOutputFormatter>::default(),
            OutputFormat::JUnit => Box::<JUnitOutputFormatter>::default(),
            OutputFormat::Sarif => Box::<SarifOutputFormatter>::default(),
        }
    }

    /// Print all available rules by oxlint
    /// See [`InternalFormatter::all_rules`] for more details.
    pub fn all_rules(&self, enabled_rules: FxHashSet<&str>) -> Option<String> {
        self.internal.all_rules(enabled_rules)
    }

    /// At the end of the Lint command we may output extra information.
    pub fn lint_command_info(&self, lint_command_info: &LintCommandInfo) -> Option<String> {
        if let Some(additional) = &self.additional
            && let Some(output) = additional.internal.lint_command_info(lint_command_info)
        {
            additional.output.borrow_mut().push_str(&output);
        }
        self.internal.lint_command_info(lint_command_info)
    }

    /// Returns the [`DiagnosticReporter`] which then will be used by [`DiagnosticService`](oxc_diagnostics::DiagnosticService)
    /// See [`InternalFormatter::get_diagnostic_reporter`] for more details.
    pub fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        let primary = self.internal.get_diagnostic_reporter();
        if let Some(additional) = &self.additional {
            Box::new(AdditionalOutputReporter {
                primary,
                additional: additional.internal.get_diagnostic_reporter(),
                output: Rc::clone(&additional.output),
                silent_primary: additional.silent_primary,
            })
        } else {
            primary
        }
    }

    pub fn take_additional_output(&self) -> Option<String> {
        self.additional.as_ref().map(|additional| {
            let mut output = additional.output.borrow_mut();
            std::mem::take(&mut *output)
        })
    }
}

struct AdditionalOutputReporter {
    primary: Box<dyn DiagnosticReporter>,
    additional: Box<dyn DiagnosticReporter>,
    output: Rc<RefCell<String>>,
    silent_primary: bool,
}

impl DiagnosticReporter for AdditionalOutputReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        if let Some(output) = self.additional.finish(result) {
            self.output.borrow_mut().push_str(&output);
        }

        self.primary.finish(result)
    }

    fn supports_minified_file_fallback(&self) -> bool {
        // The additional formatter must receive every original diagnostic. The service-level
        // fallback replaces diagnostics based on primary output and can make the file incomplete.
        false
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let error: Arc<dyn Diagnostic + Send + Sync> = error.into();
        let primary_output = if self.silent_primary {
            None
        } else {
            self.primary.render_error(Box::new(SharedDiagnostic(Arc::clone(&error))))
        };

        if let Some(output) = self.additional.render_error(Box::new(SharedDiagnostic(error))) {
            self.output.borrow_mut().push_str(&output);
        }

        primary_output
    }
}

struct SharedDiagnostic(Arc<dyn Diagnostic + Send + Sync>);

impl Debug for SharedDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.0.as_ref(), formatter)
    }
}

impl Display for SharedDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.0.as_ref(), formatter)
    }
}

impl StdError for SharedDiagnostic {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Diagnostic for SharedDiagnostic {
    fn code(&self) -> Option<Cow<'_, str>> {
        self.0.code()
    }

    fn severity(&self) -> Option<oxc_diagnostics::Severity> {
        self.0.severity()
    }

    fn help(&self) -> Option<Cow<'_, str>> {
        self.0.help()
    }

    fn note(&self) -> Option<Cow<'_, str>> {
        self.0.note()
    }

    fn url(&self) -> Option<Cow<'_, str>> {
        self.0.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.0.source_code()
    }

    fn labels(&self) -> &[LabeledSpan] {
        self.0.labels()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{DiagnosticService, NamedSource, OxcDiagnostic};
    use oxc_linter::OxlintSuppressionFileAction;
    use oxc_span::Span;

    use crate::{
        output_formatter::{LintCommandInfo, OutputFormat, OutputFormatter},
        tester::Tester,
    };

    const TEST_CWD: &str = "fixtures/cli/output_formatter_diagnostic";

    #[test]
    fn writes_the_same_diagnostic_in_two_formats() {
        let formatter = OutputFormatter::new_with_additional_output(
            OutputFormat::Default,
            OutputFormat::Json,
            false,
        );
        let (mut service, sender) = DiagnosticService::new(formatter.get_diagnostic_reporter());
        sender
            .send(vec![
                OxcDiagnostic::warn("test warning")
                    .with_label(Span::new(0, 8))
                    .with_source_code(NamedSource::new("test.js", "debugger;")),
            ])
            .unwrap();
        drop(sender);

        let mut stdout = Vec::new();
        service.run(&mut stdout);
        let info = LintCommandInfo {
            number_of_files: 1,
            number_of_rules: Some(1),
            threads_count: 1,
            start_time: Duration::ZERO,
            oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
            rule_timings: None,
        };
        stdout.extend(formatter.lint_command_info(&info).unwrap().bytes());

        let stdout = String::from_utf8(stdout).unwrap();
        assert!(stdout.contains("test warning"));

        let additional = formatter.take_additional_output().unwrap();
        let report: serde_json::Value = serde_json::from_str(&additional).unwrap();
        let diagnostics = report["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["message"], "test warning");
    }

    #[test]
    fn test_output_formatter_diagnostic_formats() {
        let mut formats: Vec<&str> =
            vec!["checkstyle", "default", "github", "junit", "agent", "stylish", "unix", "sarif"];

        // disabled for windows
        // json will output the offset which will be different for windows
        // when there are multiple lines (`\r\n` vs `\n`)
        if cfg!(not(target_os = "windows")) {
            formats.push("json");
        }

        // Exclude `gitlab` on big-endian systems because fingerprints differ there
        if cfg!(not(target_endian = "big")) {
            formats.push("gitlab");
        }

        for fmt in &formats {
            let args_vec = [format!("--format={fmt}"), "test.js".to_string()];
            let args_ref: Vec<&str> = args_vec.iter().map(std::string::String::as_str).collect();
            Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(&args_ref);
        }
    }

    #[test]
    fn test_output_formatter_diagnostic_formats_success() {
        let mut formats: Vec<&str> =
            vec!["checkstyle", "default", "github", "junit", "agent", "stylish", "unix", "sarif"];

        // disabled for windows
        // json will output the offset which will be different for windows
        // when there are multiple lines (`\r\n` vs `\n`)
        if cfg!(not(target_os = "windows")) {
            formats.push("json");
        }

        // Exclude `gitlab` on big-endian systems because fingerprints differ there
        if cfg!(not(target_endian = "big")) {
            formats.push("gitlab");
        }

        for fmt in &formats {
            let args_vec = [format!("--format={fmt}"), "ok.js".to_string()];
            let args_ref: Vec<&str> = args_vec.iter().map(std::string::String::as_str).collect();
            Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(&args_ref);
        }
    }

    // Regression test for https://github.com/oxc-project/oxc/issues/19588
    // Parser errors with colons in their message (e.g. 'Expected `;` but found `:`')
    // were being truncated to just the character after the first colon.
    #[test]
    fn test_output_formatter_diagnostic_formats_with_parser_error() {
        let mut formats: Vec<&str> =
            vec!["checkstyle", "default", "github", "junit", "agent", "stylish", "unix", "sarif"];

        // disabled for windows
        // json will output the offset which will be different for windows
        // when there are multiple lines (`\r\n` vs `\n`)
        if cfg!(not(target_os = "windows")) {
            formats.push("json");
        }

        // Exclude `gitlab` on big-endian systems because fingerprints differ there
        if cfg!(not(target_endian = "big")) {
            formats.push("gitlab");
        }

        for fmt in &formats {
            let args_vec = [format!("--format={fmt}"), "parser-error.js".to_string()];
            let args_ref: Vec<&str> = args_vec.iter().map(std::string::String::as_str).collect();
            Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(&args_ref);
        }
    }

    // Test that each of the formatters can output the disable directive violations.
    #[test]
    fn test_output_formatter_diagnostic_formats_with_disable_directive() {
        let mut formats: Vec<&str> =
            vec!["checkstyle", "default", "github", "junit", "agent", "stylish", "unix", "sarif"];

        // disabled for windows
        // json will output the offset which will be different for windows
        // when there are multiple lines (`\r\n` vs `\n`)
        if cfg!(not(target_os = "windows")) {
            formats.push("json");
        }

        // Exclude `gitlab` on big-endian systems because fingerprints differ there
        if cfg!(not(target_endian = "big")) {
            formats.push("gitlab");
        }

        for fmt in &formats {
            let args_vec = [
                format!("--format={fmt}"),
                "--report-unused-disable-directives".to_string(),
                "disable-directive.js".to_string(),
            ];
            let args_ref: Vec<&str> = args_vec.iter().map(std::string::String::as_str).collect();
            Tester::new().with_cwd(TEST_CWD.into()).test_and_snapshot(&args_ref);
        }
    }
}

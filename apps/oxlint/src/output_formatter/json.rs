use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use oxc_str::CompactStr;

use cow_utils::CowUtils;
use miette::JSONReportHandler;
use rustc_hash::FxHashSet;
use serde::Serialize;
use url::Url;

use oxc_diagnostics::{
    Error,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{RuleCategory, rules::RULES};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct JsonOutputFormatter {
    cwd: PathBuf,
    reporter: JsonReporterWrapper,
}

impl JsonOutputFormatter {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd, reporter: JsonReporterWrapper::default() }
    }
}

impl InternalFormatter for JsonOutputFormatter {
    fn all_rules(&self, _enabled_rules: FxHashSet<&str>) -> Option<String> {
        #[derive(Debug, Serialize)]
        struct RuleInfoJson<'a> {
            scope: &'a str,
            value: &'a str,
            category: RuleCategory,
            #[cfg(feature = "ruledocs")]
            version: &'a str,
            type_aware: bool,
            fix: String,
            default: bool,
            docs_url: CompactStr,
        }

        // Determine which rules are turned on by default (same logic as RuleTable)
        let default_plugin_names = ["eslint", "unicorn", "typescript", "oxc"];
        let default_rules: FxHashSet<&'static str> = RULES
            .iter()
            .filter(|rule| {
                rule.category() == RuleCategory::Correctness
                    && default_plugin_names.contains(&rule.plugin_name())
            })
            .map(oxc_linter::rules::RuleEnum::name)
            .collect();

        let mut rules_info: Vec<_> = RULES
            .iter()
            .map(|rule| RuleInfoJson {
                scope: rule.plugin_name(),
                value: rule.name(),
                category: rule.category(),
                #[cfg(feature = "ruledocs")]
                version: rule.version(),
                type_aware: rule.is_tsgolint_rule(),
                fix: rule.fix().to_string(),
                default: default_rules.contains(rule.name()),
                docs_url: format!(
                    "https://oxc.rs/docs/guide/usage/linter/rules/{}/{}.html",
                    rule.plugin_name(),
                    rule.name()
                )
                .into(),
            })
            .collect();

        rules_info.sort_by_key(|rule| (rule.scope, rule.value));

        Some(serde_json::to_string_pretty(&rules_info).expect("Failed to serialize"))
    }

    fn lint_command_info(&self, lint_command_info: &super::LintCommandInfo) -> Option<String> {
        let diagnostics = self.reporter.0.borrow_mut().render(&self.cwd);
        let number_of_rules =
            lint_command_info.number_of_rules.map_or("null".to_string(), |x| x.to_string());
        let start_time = lint_command_info.start_time.as_secs_f64();

        Some(format!(
            r#"{{ "diagnostics": {},
              "number_of_files": {},
              "number_of_rules": {},
              "threads_count": {},
              "start_time": {}
            }}
            "#,
            diagnostics,
            lint_command_info.number_of_files,
            number_of_rules,
            lint_command_info.threads_count,
            start_time,
        ))
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(self.reporter.clone())
    }
}

/// Renders reports as a JSON array of objects.
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
#[derive(Default, Debug)]
struct JsonReporter {
    diagnostics: Vec<Error>,
}

#[derive(Clone, Debug, Default)]
pub struct JsonReporterWrapper(Rc<RefCell<JsonReporter>>);

impl DiagnosticReporter for JsonReporterWrapper {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.0.borrow_mut().render_error(error)
    }
}

impl DiagnosticReporter for JsonReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

impl JsonReporter {
    pub(super) fn render(&mut self, cwd: &Path) -> String {
        format_json(&mut self.diagnostics, cwd)
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-json>
fn format_json(diagnostics: &mut Vec<Error>, cwd: &Path) -> String {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| {
            let mut output = String::new();
            handler.render_report(&mut output, error.as_ref()).unwrap();
            make_filename_relative(&mut output, cwd);
            output
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[{messages}]")
}

fn make_filename_relative(output: &mut String, cwd: &Path) {
    if !output.contains(r#""filename": "file:"#) {
        return;
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(output) else {
        return;
    };
    let Some(filename) = value.get("filename").and_then(serde_json::Value::as_str) else {
        return;
    };
    let Ok(url) = Url::parse(filename) else {
        return;
    };
    let Ok(path) = url.to_file_path() else {
        return;
    };

    let relative_path = path.strip_prefix(cwd).unwrap_or(&path).to_string_lossy();
    let relative_path = relative_path.cow_replace('\\', "/");
    let original = serde_json::to_string(filename).unwrap();
    let replacement = serde_json::to_string(&relative_path).unwrap();
    *output = output
        .cow_replacen(
            &format!(r#""filename": {original}"#),
            &format!(r#""filename": {replacement}"#),
            1,
        )
        .into_owned();
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticResult};
    use oxc_span::Span;
    use url::Url;

    use crate::output_formatter::{
        InternalFormatter, LintCommandInfo, OxlintSuppressionFileAction, json::JsonOutputFormatter,
    };

    #[test]
    fn reporter() {
        let cwd = tempfile::tempdir().unwrap();
        let filename = Url::from_file_path(cwd.path().join("test file.ts")).unwrap();
        let formatter = JsonOutputFormatter::new(cwd.path().to_path_buf());

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new(filename, "debugger;"));

        let mut diagnostic_reporter = formatter.get_diagnostic_reporter();
        let first_result = diagnostic_reporter.render_error(error);

        // reporter keeps it in memory
        assert!(first_result.is_none());

        // report not gives us all diagnostics at ones
        let second_result = diagnostic_reporter.finish(&DiagnosticResult::default());

        assert!(second_result.is_none());
        let output = formatter
            .lint_command_info(&LintCommandInfo {
                number_of_files: 0,
                number_of_rules: Some(0),
                start_time: Duration::new(0, 0),
                threads_count: 1,
                oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
                rule_timings: None,
            })
            .unwrap();
        assert_eq!(
            &output,
            "{ \"diagnostics\": [{\"message\": \"error message\",\"severity\": \"warning\",\"causes\": [],\"filename\": \"test file.ts\",\"labels\": [{\"span\": {\"offset\": 0,\"length\": 8,\"line\": 1,\"column\": 1}}],\"related\": []}],\n              \"number_of_files\": 0,\n              \"number_of_rules\": 0,\n              \"threads_count\": 1,\n              \"start_time\": 0\n            }\n            "
        );
    }
}

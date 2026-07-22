use std::{cell::RefCell, rc::Rc};

use oxc_str::CompactStr;

use miette::JSONReportHandler;
use rustc_hash::FxHashSet;
use serde::Serialize;

use oxc_diagnostics::{
    Error,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{RuleCategory, RuleTimingRecord, rules::RULES};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct JsonOutputFormatter {
    reporter: JsonReporterWrapper,
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
        let diagnostics = self.reporter.0.borrow_mut().render();
        let number_of_rules =
            lint_command_info.number_of_rules.map_or("null".to_string(), |x| x.to_string());
        let start_time = lint_command_info.start_time.as_secs_f64();
        let rule_timings = lint_command_info.rule_timings.as_ref().map_or_else(String::new, |t| {
            format!(
                ",
              \"rule_timings\": {}",
                format_rule_timings_json(t)
            )
        });

        Some(format!(
            r#"{{ "diagnostics": {},
              "number_of_files": {},
              "number_of_rules": {},
              "threads_count": {},
              "start_time": {}{}
            }}
            "#,
            diagnostics,
            lint_command_info.number_of_files,
            number_of_rules,
            lint_command_info.threads_count,
            start_time,
            rule_timings,
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
    pub(super) fn render(&mut self) -> String {
        format_json(&mut self.diagnostics)
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-json>
fn format_rule_timings_json(rule_timings: &[RuleTimingRecord]) -> String {
    #[derive(Debug, Serialize)]
    struct RuleTimingJson<'a> {
        plugin_name: &'a str,
        rule_name: &'a str,
        time_ms: f64,
        calls: u64,
        source: &'static str,
    }

    let records = rule_timings
        .iter()
        .map(|record| RuleTimingJson {
            plugin_name: &record.plugin_name,
            rule_name: &record.rule_name,
            time_ms: record.duration.as_secs_f64() * 1000.0,
            calls: record.calls,
            source: record.source.as_str(),
        })
        .collect::<Vec<_>>();

    serde_json::to_string(&records).expect("Failed to serialize rule timings")
}

fn format_json(diagnostics: &mut Vec<Error>) -> String {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| {
            let mut output = String::new();
            handler.render_report(&mut output, error.as_ref()).unwrap();
            output
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[{messages}]")
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticResult};
    use oxc_linter::{RuleTimingRecord, RuleTimingSource};
    use oxc_span::Span;

    use crate::output_formatter::{
        InternalFormatter, LintCommandInfo, OxlintSuppressionFileAction, json::JsonOutputFormatter,
    };

    #[test]
    fn reporter() {
        let formatter = JsonOutputFormatter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

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
            "{ \"diagnostics\": [{\"message\": \"error message\",\"severity\": \"warning\",\"causes\": [],\"filename\": \"file://test.ts\",\"labels\": [{\"span\": {\"offset\": 0,\"length\": 8,\"line\": 1,\"column\": 1}}],\"related\": []}],\n              \"number_of_files\": 0,\n              \"number_of_rules\": 0,\n              \"threads_count\": 1,\n              \"start_time\": 0\n            }\n            "
        );
    }

    #[test]
    fn lint_command_info_shows_rule_timings() {
        let formatter = JsonOutputFormatter::default();

        // Consume the empty diagnostics so `lint_command_info` renders them as `[]`.
        let mut diagnostic_reporter = formatter.get_diagnostic_reporter();
        diagnostic_reporter.finish(&DiagnosticResult::default());

        let output = formatter
            .lint_command_info(&LintCommandInfo {
                number_of_files: 1,
                number_of_rules: Some(2),
                start_time: Duration::from_millis(5),
                threads_count: 1,
                oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
                rule_timings: Some(vec![
                    RuleTimingRecord {
                        source: RuleTimingSource::Native,
                        plugin_name: "eslint".to_string(),
                        rule_name: "no-debugger".to_string(),
                        duration: Duration::from_micros(1500),
                        calls: 3,
                    },
                    RuleTimingRecord {
                        source: RuleTimingSource::TypeAware,
                        plugin_name: "typescript".to_string(),
                        rule_name: "no-floating-promises".to_string(),
                        duration: Duration::from_micros(500),
                        calls: 0,
                    },
                ]),
            })
            .unwrap();
        assert_eq!(
            &output,
            "{ \"diagnostics\": [],\n              \"number_of_files\": 1,\n              \"number_of_rules\": 2,\n              \"threads_count\": 1,\n              \"start_time\": 0.005,\n              \"rule_timings\": [{\"plugin_name\":\"eslint\",\"rule_name\":\"no-debugger\",\"time_ms\":1.5,\"calls\":3,\"source\":\"native\"},{\"plugin_name\":\"typescript\",\"rule_name\":\"no-floating-promises\",\"time_ms\":0.5,\"calls\":0,\"source\":\"type-aware\"}]\n            }\n            "
        );
    }
}

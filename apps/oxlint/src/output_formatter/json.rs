use std::{cell::RefCell, rc::Rc};

use oxc_str::CompactStr;

use miette::{Diagnostic, JSONReportHandler, SourceCode, SourceSpan, SpanContents};
use rustc_hash::FxHashSet;
use serde::Serialize;
use serde_json::Value;

use oxc_diagnostics::{
    Error,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{RuleCategory, rules::RULES};

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
    pub(super) fn render(&mut self) -> String {
        format_json(&mut self.diagnostics)
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-json>
fn format_json(diagnostics: &mut Vec<Error>) -> String {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| format_diagnostic_json(&handler, error.as_ref()))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[{messages}]")
}

fn format_diagnostic_json(handler: &JSONReportHandler, diagnostic: &dyn Diagnostic) -> String {
    let mut output = String::new();
    handler.render_report(&mut output, diagnostic).unwrap();

    let Ok(mut output_json) = serde_json::from_str::<Value>(&output) else {
        return output;
    };

    add_label_end_positions(&mut output_json, diagnostic, None);
    serde_json::to_string(&output_json).unwrap_or(output)
}

fn add_label_end_positions(
    output_json: &mut Value,
    diagnostic: &dyn Diagnostic,
    parent_src: Option<&dyn SourceCode>,
) {
    let src = diagnostic.source_code().or(parent_src);

    if let Some(src) = src
        && let Some(labels_json) = output_json.get_mut("labels").and_then(Value::as_array_mut)
    {
        let labels = diagnostic.labels();

        for (label_json, label) in labels_json.iter_mut().zip(labels.iter()) {
            let Some(span_json) = label_json.get_mut("span").and_then(Value::as_object_mut) else {
                continue;
            };
            if span_json.get("line").is_none_or(Value::is_null)
                || span_json.get("column").is_none_or(Value::is_null)
            {
                continue;
            }

            let end_offset = label.inner().offset() + label.inner().len();
            let end_span = SourceSpan::from((end_offset, 0));
            let Ok(end) = src.read_span(&end_span, 0, 0) else {
                continue;
            };

            span_json.insert("endLine".to_string(), Value::from(end.line() + 1));
            span_json.insert("endColumn".to_string(), Value::from(end.column() + 1));
        }
    }

    let related = diagnostic.related();
    let Some(related_json) = output_json.get_mut("related").and_then(Value::as_array_mut) else {
        return;
    };

    for (related_json, related_diagnostic) in related_json.iter_mut().zip(related.iter()) {
        add_label_end_positions(related_json, *related_diagnostic, src);
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticResult};
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
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        let span = &json["diagnostics"][0]["labels"][0]["span"];

        assert_eq!(json["number_of_files"], 0);
        assert_eq!(json["number_of_rules"], 0);
        assert_eq!(json["threads_count"], 1);
        assert_eq!(json["start_time"], 0);
        assert_eq!(json["diagnostics"][0]["message"], "error message");
        assert_eq!(json["diagnostics"][0]["severity"], "warning");
        assert_eq!(json["diagnostics"][0]["filename"], "file://test.ts");
        assert_eq!(span["offset"], 0);
        assert_eq!(span["length"], 8);
        assert_eq!(span["line"], 1);
        assert_eq!(span["column"], 1);
        assert_eq!(span["endLine"], 1);
        assert_eq!(span["endColumn"], 9);
    }
}

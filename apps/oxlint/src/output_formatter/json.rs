use std::{cell::RefCell, rc::Rc};

use miette::JSONReportHandler;
use serde::Serialize;

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
    fn all_rules(&self) -> Option<String> {
        #[derive(Debug, Serialize)]
        struct RuleInfoJson<'a> {
            scope: &'a str,
            value: &'a str,
            category: RuleCategory,
        }

        let rules_info = RULES.iter().map(|rule| RuleInfoJson {
            scope: rule.plugin_name(),
            value: rule.name(),
            category: rule.category(),
        });

        Some(
            serde_json::to_string_pretty(&rules_info.collect::<Vec<_>>())
                .expect("Failed to serialize"),
        )
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
    use oxc_span::Span;

    use crate::output_formatter::{InternalFormatter, LintCommandInfo, json::JsonOutputFormatter};

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
            })
            .unwrap();
        assert_eq!(
            &output,
            "{ \"diagnostics\": [{\"message\": \"error message\",\"severity\": \"warning\",\"causes\": [],\"filename\": \"file://test.ts\",\"labels\": [{\"span\": {\"offset\": 0,\"length\": 8,\"line\": 1,\"column\": 1}}],\"related\": []}],\n              \"number_of_files\": 0,\n              \"number_of_rules\": 0,\n              \"threads_count\": 1,\n              \"start_time\": 0\n            }\n            "
        );
    }
}

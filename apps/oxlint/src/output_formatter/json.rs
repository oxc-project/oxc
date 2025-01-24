use oxc_diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult},
    Error,
};
use oxc_linter::{rules::RULES, RuleCategory};

use miette::JSONReportHandler;

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct JsonOutputFormatter;

impl InternalFormatter for JsonOutputFormatter {
    fn all_rules(&self) -> Option<String> {
        #[derive(Debug, serde::Serialize)]
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

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(JsonReporter::default())
    }
}

/// Renders reports as a JSON array of objects.
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
#[derive(Default)]
struct JsonReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for JsonReporter {
    // NOTE: this output does not conform to eslint json format yet
    // https://eslint.org/docs/latest/use/formatters/#json
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_json(&mut self.diagnostics))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-json>
#[allow(clippy::print_stdout)]
fn format_json(diagnostics: &mut Vec<Error>) -> String {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| {
            let mut output = String::from("\t");
            handler.render_report(&mut output, error.as_ref()).unwrap();
            output
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[\n{messages}\n]\n")
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{
        reporter::{DiagnosticReporter, DiagnosticResult},
        NamedSource, OxcDiagnostic,
    };
    use oxc_span::Span;

    use super::JsonReporter;

    #[test]
    fn reporter() {
        let mut reporter = JsonReporter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let first_result = reporter.render_error(error);

        // reporter keeps it in memory
        assert!(first_result.is_none());

        // report not gives us all diagnostics at ones
        let second_result = reporter.finish(&DiagnosticResult::default());

        assert!(second_result.is_some());
        assert_eq!(
            second_result.unwrap(),
            "[\n\t{\"message\": \"error message\",\"severity\": \"warning\",\"causes\": [],\"filename\": \"file://test.ts\",\"labels\": [{\"span\": {\"offset\": 0,\"length\": 8}}],\"related\": []}\n]\n"
        );
    }
}

use std::io::Write;

use oxc_diagnostics::{reporter::DiagnosticReporter, Error};
use oxc_linter::rules::RULES;
use oxc_linter::RuleCategory;

use miette::JSONReportHandler;

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct JsonOutputFormatter;

impl InternalFormatter for JsonOutputFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write) {
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

        writer
            .write_all(
                serde_json::to_string_pretty(&rules_info.collect::<Vec<_>>())
                    .expect("Failed to serialize")
                    .as_bytes(),
            )
            .unwrap();
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
    fn finish(&mut self) -> Option<String> {
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
    format!("[\n{messages}\n]")
}

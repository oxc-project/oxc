use miette::JSONReportHandler;

use super::DiagnosticReporter;
use crate::Error;

/// Renders reports as a JSON array of objects.
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
#[derive(Default)]
pub struct JsonReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for JsonReporter {
    // NOTE: this output does not conform to eslint json format yet
    // https://eslint.org/docs/latest/use/formatters/#json
    fn finish(&mut self) {
        format_json(&mut self.diagnostics);
    }

    fn render_diagnostics(&mut self, _s: &[u8]) {}

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-json>
#[allow(clippy::print_stdout)]
fn format_json(diagnostics: &mut Vec<Error>) {
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
    println!("[\n{messages}\n]");
}

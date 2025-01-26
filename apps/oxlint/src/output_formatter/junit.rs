use oxc_diagnostics::{reporter::{DiagnosticReporter, DiagnosticResult}, Error};

use super::InternalFormatter;

#[derive(Default)]
pub struct JUnitOutputFormatter;

impl InternalFormatter for JUnitOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(JUnitReporter::default())
    }
}

#[derive(Default)]
struct JUnitReporter {
}

impl DiagnosticReporter for JUnitReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        todo!()
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        todo!()
    }
}
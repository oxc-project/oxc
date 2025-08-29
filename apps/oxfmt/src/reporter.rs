use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

#[derive(Debug)]
pub struct DefaultReporter;

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        let prefix = match error.severity().unwrap_or_default() {
            Severity::Error => "[ERROR] ",
            Severity::Warning => "[WARN] ",
            Severity::Advice => "",
        };
        if error.labels().is_some() {
            // TODO: GraphicalReporter should be used
            return Some(format!("{prefix}{error:?}\n"));
        }
        Some(format!("{prefix}{error}\n"))
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }
}

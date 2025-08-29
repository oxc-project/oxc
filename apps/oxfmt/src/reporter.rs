use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

#[derive(Debug)]
pub struct DefaultReporter;

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        if error.labels().is_some() {
            // TODO: GraphicalReporter should be used
            return Some(format!("{error:?}\n"));
        }

        let prefix = match error.severity().unwrap_or_default() {
            Severity::Warning => "[warn] ",
            _ => "",
        };
        // NOTE: Formatted `path` should be inlined in `error`,
        // there is no way to get here since we do not have `labels`
        Some(format!("{prefix}{error}\n"))
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }
}

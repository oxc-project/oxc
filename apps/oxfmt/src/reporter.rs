use oxc_diagnostics::{
    Error, GraphicalReportHandler, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

#[derive(Debug)]
pub struct DefaultReporter {
    handler: GraphicalReportHandler,
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new() }
    }
}

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        // If `.labels()` exists, it means this comes from `oxc_parser`, `oxc_formatter`, etc
        if error.labels().is_some() {
            let mut output = String::new();
            self.handler.render_report(&mut output, error.as_ref()).unwrap();
            return Some(output);
        }

        // Otherwise, this is a error without `labels`, originate from `oxfmt` itself
        let prefix = match error.severity().unwrap_or_default() {
            Severity::Warning => "[warn] ",
            _ => "",
        };
        // NOTE: Formatted `path` should be inlined
        // there is no way to get here since we do not have `labels`
        Some(format!("{prefix}{error}\n"))
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        // Leave it to the caller with `diagnostic_service.run()`, to determine exit code
        None
    }
}

use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

// This reporter is used with stderr and displays diagnostics only in a graphical way.
// For stdout, we display them manually in `format.rs`.

#[derive(Debug)]
pub struct DefaultReporter {
    handler: GraphicalReportHandler,
    diagnostics: Vec<Error>,
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new(), diagnostics: Vec::new() }
    }
}

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        // Collect diagnostics for rendering in finish() at once
        self.diagnostics.push(error);
        None
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        let mut output = String::new();

        // Render all diagnostics (errors only, no warnings)
        for diagnostic in &self.diagnostics {
            self.handler.render_report(&mut output, diagnostic.as_ref()).unwrap();
        }

        Some(output)
    }
}

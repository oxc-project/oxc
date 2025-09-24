use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

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
        // Collect diagnostics for sorting instead of immediate rendering
        self.diagnostics.push(error);
        None
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        let mut output = String::new();

        // Sort diagnostics: labels-less warnings first, then errors with labels last
        self.diagnostics.sort_by_cached_key(|diagnostic| {
            if diagnostic.labels().is_none() {
                (0, diagnostic.to_string())
            } else {
                (1, diagnostic.to_string())
            }
        });

        // Render all sorted diagnostics
        for diagnostic in &self.diagnostics {
            // If `.labels()` exists, it means this comes from `oxc_parser`, `oxc_formatter`, etc
            // Otherwise, this is a warnings just contains formatted path from `oxfmt` itself
            if diagnostic.labels().is_none() {
                output.push_str(format!("{diagnostic}\n").as_str());
            } else {
                self.handler.render_report(&mut output, diagnostic.as_ref()).unwrap();
            }
        }

        Some(output)
    }
}

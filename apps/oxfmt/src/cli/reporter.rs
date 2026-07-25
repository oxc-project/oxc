use std::path::PathBuf;

use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
    with_file_url,
};

// This reporter is used with stderr and displays diagnostics only in a graphical way.
// For stdout, we display them manually in `format.rs`.

#[derive(Debug)]
pub struct DefaultReporter {
    handler: GraphicalReportHandler,
    diagnostics: Vec<Error>,
    jetbrains_cwd: Option<PathBuf>,
}

impl DefaultReporter {
    pub fn new(cwd: PathBuf) -> Self {
        let is_jetbrains =
            std::env::var("TERMINAL_EMULATOR").is_ok_and(|value| value == "JetBrains-JediTerm");
        Self {
            handler: GraphicalReportHandler::new(),
            diagnostics: Vec::new(),
            jetbrains_cwd: is_jetbrains.then_some(cwd),
        }
    }
}

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        // Collect diagnostics for rendering in finish() at once
        let error =
            if let Some(cwd) = &self.jetbrains_cwd { with_file_url(error, cwd) } else { error };
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

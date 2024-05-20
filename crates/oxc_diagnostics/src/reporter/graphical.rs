use std::io::{BufWriter, Stdout, Write};

use crate::{Error, GraphicalReportHandler};

use super::{writer, DiagnosticReporter};

pub struct GraphicalReporter {
    handler: GraphicalReportHandler,
    writer: BufWriter<Stdout>,
}

impl Default for GraphicalReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new(), writer: writer() }
    }
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self) {
        self.writer.flush().unwrap();
    }

    fn render_diagnostics(&mut self, s: &[u8]) {
        self.writer.write_all(s).unwrap();
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}

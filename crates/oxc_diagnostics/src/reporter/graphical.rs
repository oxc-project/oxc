use std::io::{BufWriter, ErrorKind, Stdout, Write};

use super::DiagnosticReporter;
use crate::{Error, GraphicalReportHandler};

/// Pretty-prints diagnostics. Primarily meant for human-readable output in a terminal.
///
/// See [`GraphicalReportHandler`] for how to configure colors, context lines, etc.
pub struct GraphicalReporter {
    handler: GraphicalReportHandler,
}

impl Default for GraphicalReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new() }
    }
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self, writer: &mut BufWriter<Stdout>) {
        writer
            .flush()
            .or_else(|e| {
                // Do not panic when the process is skill (e.g. piping into `less`).
                if matches!(e.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .unwrap();
    }

    fn render_diagnostics(&mut self, writer: &mut BufWriter<Stdout>, s: &[u8]) {
        writer
            .write_all(s)
            .or_else(|e| {
                // Do not panic when the process is skill (e.g. piping into `less`).
                if matches!(e.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .unwrap();
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}

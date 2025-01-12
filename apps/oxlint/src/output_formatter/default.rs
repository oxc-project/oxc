use std::io::{BufWriter, ErrorKind, Stdout, Write};

use oxc_diagnostics::{reporter::DiagnosticReporter, Error, GraphicalReportHandler};
use oxc_linter::table::RuleTable;

use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        let table = RuleTable::new();
        for section in table.sections {
            writeln!(writer, "{}", section.render_markdown_table(None)).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GraphicalReporter::default())
    }
}

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

#[cfg(test)]
mod test {
    use crate::output_formatter::{default::DefaultOutputFormatter, InternalFormatter};
    use std::io::BufWriter;

    #[test]
    fn all_rules() {
        let mut writer = BufWriter::new(std::io::stdout());
        let mut formatter = DefaultOutputFormatter;

        formatter.all_rules(&mut writer);
        assert!(!writer.buffer().is_empty());
    }
}

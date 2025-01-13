use std::io::Write;

use oxc_diagnostics::{reporter::DiagnosticReporter, Error, GraphicalReportHandler};
use oxc_linter::table::RuleTable;

use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write) {
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
struct GraphicalReporter {
    handler: GraphicalReportHandler,
}

impl Default for GraphicalReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new() }
    }
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self) -> Option<String> {
        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}

#[cfg(test)]
mod test {
    use crate::output_formatter::{
        default::{DefaultOutputFormatter, GraphicalReporter},
        InternalFormatter,
    };
    use miette::NamedSource;
    use oxc_diagnostics::{reporter::DiagnosticReporter, OxcDiagnostic};
    use oxc_span::Span;

    #[test]
    fn all_rules() {
        let mut writer = Vec::new();
        let mut formatter = DefaultOutputFormatter;

        formatter.all_rules(&mut writer);
        assert!(!writer.is_empty());
    }

    #[test]
    fn reporter_finish() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish();

        assert!(result.is_none());
    }

    #[test]
    fn reporter_error() {
        let mut reporter = GraphicalReporter::default();
        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\n  \u{1b}[38;2;244;191;117;1m⚠\u{1b}[0m \u{1b}[38;2;244;191;117;1merror message\u{1b}[0m\n   ╭─[\u{1b}[38;2;92;157;255;1mfile://test.ts\u{1b}[0m:1:1]\n \u{1b}[2m1\u{1b}[0m │ debugger;\n   · \u{1b}[38;2;246;87;248m────────\u{1b}[0m\n   ╰────\n"
        );
    }
}

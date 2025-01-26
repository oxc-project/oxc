use std::borrow::Cow;

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
    Error, Severity,
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct UnixOutputFormatter;

impl InternalFormatter for UnixOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(UnixReporter::default())
    }
}

/// Reporter to output diagnostics in a simple one line output.
/// At the end it reports the total numbers of diagnostics.
#[derive(Default)]
struct UnixReporter {
    total: usize,
}

impl DiagnosticReporter for UnixReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        let total = self.total;
        if total > 0 {
            return Some(format!("\n{total} problem{}\n", if total > 1 { "s" } else { "" }));
        }

        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.total += 1;
        Some(format_unix(&error))
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-unix>
fn format_unix(diagnostic: &Error) -> String {
    let Info { start, end: _, filename, message, severity, rule_id } = Info::new(diagnostic);
    let severity = match severity {
        Severity::Error => "Error",
        _ => "Warning",
    };
    let rule_id =
        rule_id.map_or_else(|| Cow::Borrowed(""), |rule_id| Cow::Owned(format!("/{rule_id}")));
    format!("{filename}:{}:{}: {message} [{severity}{rule_id}]\n", start.line, start.column)
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{
        reporter::{DiagnosticReporter, DiagnosticResult},
        NamedSource, OxcDiagnostic,
    };
    use oxc_span::Span;

    use super::UnixReporter;

    #[test]
    fn reporter_finish_empty() {
        let mut reporter = UnixReporter::default();

        let result = reporter.finish(&DiagnosticResult::default());

        assert!(result.is_none());
    }

    #[test]
    fn reporter_finish_one_entry() {
        let mut reporter = UnixReporter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let _ = reporter.render_error(error);
        let result = reporter.finish(&DiagnosticResult::default());

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "\n1 problem\n");
    }

    #[test]
    fn reporter_error() {
        let mut reporter = UnixReporter::default();
        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "file://test.ts:1:1: error message [Warning]\n");
    }
}

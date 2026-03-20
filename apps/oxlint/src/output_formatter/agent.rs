use std::collections::BTreeMap;
use std::fmt::Write;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct AgentOutputFormatter;

impl InternalFormatter for AgentOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(AgentReporter::default())
    }
}

/// Reporter optimized for AI agent consumption.
/// Groups diagnostics by filename and uses a minimal plain-text format
/// to reduce token usage. Includes rule IDs and help text.
#[derive(Default)]
struct AgentReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for AgentReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        if self.diagnostics.is_empty() {
            return None;
        }

        let mut output = format_agent(&mut self.diagnostics);

        let errors = result.errors_count();
        let warnings = result.warnings_count();
        output.push('\n');
        match (errors, warnings) {
            (0, 0) => {}
            (e, 0) => {
                let _ = write!(output, "{e} error{}\n", if e == 1 { "" } else { "s" });
            }
            (0, w) => {
                let _ = write!(output, "{w} warning{}\n", if w == 1 { "" } else { "s" });
            }
            (e, w) => {
                let _ = write!(
                    output,
                    "{e} error{}, {w} warning{}\n",
                    if e == 1 { "" } else { "s" },
                    if w == 1 { "" } else { "s" }
                );
            }
        }

        Some(output)
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_agent(diagnostics: &mut Vec<Error>) -> String {
    // Group diagnostics by filename using BTreeMap for deterministic ordering
    let mut by_file: BTreeMap<String, Vec<(Info, Option<String>)>> = BTreeMap::new();

    for error in diagnostics.drain(..) {
        let help = error.help().map(|h| h.to_string());
        let info = Info::new(&error);
        by_file.entry(info.filename.clone()).or_default().push((info, help));
    }

    let mut output = String::new();

    for (filename, entries) in &by_file {
        output.push_str(filename);
        output.push('\n');

        for (info, help) in entries {
            let severity = match info.severity {
                Severity::Error => "error",
                _ => "warning",
            };
            let rule = info.rule_id.as_deref().unwrap_or("-");
            let _ = writeln!(
                output,
                "  {}:{} {} {}: {}",
                info.start.line, info.start.column, severity, rule, info.message
            );
            if let Some(help) = help {
                let _ = writeln!(output, "    help: {help}");
            }
        }
    }

    output
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{
        NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::AgentReporter;

    #[test]
    fn reporter_finish_empty() {
        let mut reporter = AgentReporter::default();
        let result = reporter.finish(&DiagnosticResult::default());
        assert!(result.is_none());
    }

    #[test]
    fn reporter_groups_by_file() {
        let mut reporter = AgentReporter::default();

        let error1 = OxcDiagnostic::error("no debugger")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "debugger;"));
        let error2 = OxcDiagnostic::warn("no console")
            .with_label(Span::new(0, 7))
            .with_source_code(NamedSource::new("file.js", "console;"));

        let _ = reporter.render_error(error1);
        let _ = reporter.render_error(error2);

        let result = reporter.finish(&DiagnosticResult::new(1, 1, false));

        assert!(result.is_some());
        let output = result.unwrap();
        // Both diagnostics should be under the same file header
        assert_eq!(output.matches("file.js\n").count(), 1);
        assert!(output.contains("error"));
        assert!(output.contains("warning"));
        assert!(output.contains("1 error, 1 warning"));
    }

    #[test]
    fn reporter_includes_help_text() {
        let mut reporter = AgentReporter::default();

        let error = OxcDiagnostic::warn("no debugger")
            .with_help("Remove the `debugger` statement")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "debugger;"));

        let _ = reporter.render_error(error);

        let result = reporter.finish(&DiagnosticResult::new(1, 0, false));

        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("help: Remove the `debugger` statement"));
    }

    #[test]
    fn reporter_error() {
        let mut reporter = AgentReporter::default();

        let error = OxcDiagnostic::warn("no debugger")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "debugger;"));

        let _ = reporter.render_error(error);

        let result = reporter.finish(&DiagnosticResult::new(1, 0, false));

        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("file.js\n"));
        assert!(output.contains("1:1 warning"));
    }
}

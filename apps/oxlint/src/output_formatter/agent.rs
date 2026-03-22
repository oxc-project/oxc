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
/// Outputs one greppable line per diagnostic in the format:
/// `file:line:col severity rule(id): message (help: ...)`
#[derive(Default)]
struct AgentReporter {
    output: String,
}

impl DiagnosticReporter for AgentReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        if self.output.is_empty() {
            return None;
        }

        let errors = result.errors_count();
        let warnings = result.warnings_count();
        self.output.push('\n');
        match (errors, warnings) {
            (0, 0) => {}
            (e, 0) => {
                let _ = writeln!(self.output, "{e} error{}", if e == 1 { "" } else { "s" });
            }
            (0, w) => {
                let _ = writeln!(self.output, "{w} warning{}", if w == 1 { "" } else { "s" });
            }
            (e, w) => {
                let _ = writeln!(
                    self.output,
                    "{e} error{}, {w} warning{}",
                    if e == 1 { "" } else { "s" },
                    if w == 1 { "" } else { "s" }
                );
            }
        }

        Some(std::mem::take(&mut self.output))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let help = error.help().map(|h| h.to_string());
        let info = Info::new(&error);
        let severity = match info.severity {
            Severity::Error => "error",
            _ => "warning",
        };
        let rule = info.rule_id.as_deref().unwrap_or("-");

        if let Some(help) = help {
            let _ = writeln!(
                self.output,
                "{}:{}:{} {} {}: {} (help: {})",
                info.filename, info.start.line, info.start.column, severity, rule, info.message, help
            );
        } else {
            let _ = writeln!(
                self.output,
                "{}:{}:{} {} {}: {}",
                info.filename, info.start.line, info.start.column, severity, rule, info.message
            );
        }

        None
    }
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
    fn reporter_one_liner_format() {
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
        // Each diagnostic on its own greppable line
        assert!(output.contains("file.js:1:1 error"));
        assert!(output.contains("file.js:1:1 warning"));
        assert!(output.contains("1 error, 1 warning"));
    }

    #[test]
    fn reporter_includes_help_inline() {
        let mut reporter = AgentReporter::default();

        let error = OxcDiagnostic::warn("no debugger")
            .with_help("Remove the `debugger` statement")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "debugger;"));

        let _ = reporter.render_error(error);

        let result = reporter.finish(&DiagnosticResult::new(1, 0, false));

        assert!(result.is_some());
        let output = result.unwrap();
        // Help text should be inline, not on a separate line
        assert!(output.contains("(help: Remove the `debugger` statement)"));
        assert!(!output.contains("\n    help:"));
    }

    #[test]
    fn reporter_greppable_line() {
        let mut reporter = AgentReporter::default();

        let error = OxcDiagnostic::warn("no debugger")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "debugger;"));

        let _ = reporter.render_error(error);

        let result = reporter.finish(&DiagnosticResult::new(1, 0, false));

        assert!(result.is_some());
        let output = result.unwrap();
        // Line should start with file:line:col for greppability
        assert!(output.starts_with("file.js:1:1 warning"));
    }
}

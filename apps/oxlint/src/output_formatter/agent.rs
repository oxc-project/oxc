use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct AgentOutputFormatter;

impl InternalFormatter for AgentOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(AgentReporter)
    }
}

/// Agent-friendly reporter: one line per diagnostic, no source excerpts, no summary.
#[derive(Default)]
struct AgentReporter;

impl DiagnosticReporter for AgentReporter {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }

    fn supports_minified_file_fallback(&self) -> bool {
        false
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        Some(format_agent(&error))
    }
}

fn format_agent(diagnostic: &Error) -> String {
    let Info { start, filename, rule_id, .. } = Info::new(diagnostic);
    let filename = if filename.is_empty() {
        diagnostic
            .source_code()
            .and_then(miette::SourceCode::name)
            .map_or_else(|| "<unknown>".to_string(), ToString::to_string)
    } else {
        filename
    };
    let severity = match diagnostic.severity() {
        Some(Severity::Warning) => "warning",
        Some(Severity::Advice) => "advice",
        _ => "error",
    };
    let rule = rule_id.map_or_else(String::new, |rule_id| format!(" {rule_id}"));
    let message = compact_message(&diagnostic.to_string());

    if start.line == 0 {
        format!("{filename}: {severity}{rule}: {message}\n")
    } else {
        format!("{filename}:{}:{}: {severity}{rule}: {message}\n", start.line, start.column)
    }
}

fn compact_message(message: &str) -> String {
    let mut compact = String::new();
    for word in message.split_whitespace() {
        if !compact.is_empty() {
            compact.push(' ');
        }
        compact.push_str(word);
    }
    compact
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticReporter};
    use oxc_span::Span;

    use super::AgentReporter;

    #[test]
    fn reporter_error() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::warn("error message")
            .with_error_code("eslint", "no-debugger")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert_eq!(
            result.unwrap(),
            "file://test.ts:1:1: warning eslint(no-debugger): error message\n"
        );
    }

    #[test]
    fn reporter_error_message_with_colon() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::error("Expected `;` but found `:`")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("file://test.js", ":"));

        let result = reporter.render_error(error);

        assert_eq!(result.unwrap(), "file://test.js:1:1: error: Expected `;` but found `:`\n");
    }

    #[test]
    fn reporter_error_without_label() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::error("Failed to parse\nconfiguration")
            .with_source_code(NamedSource::new("config.json", ""));

        let result = reporter.render_error(error);

        assert_eq!(result.unwrap(), "config.json: error: Failed to parse configuration\n");
    }
}

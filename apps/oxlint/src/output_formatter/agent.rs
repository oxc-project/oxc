use std::borrow::Cow;

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
    let Info { start, filename, message: info_message, rule_id, .. } = Info::new(diagnostic);
    let filename = if filename.is_empty() {
        diagnostic
            .source_code()
            .and_then(|source| source.name())
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
    // `Info` only fills in the message when the diagnostic has a resolvable label.
    let rendered_message =
        if info_message.is_empty() { diagnostic.to_string() } else { info_message };
    let message = compact_message(&rendered_message);
    let help = diagnostic
        .help()
        .map(|help| format!(" help: {}", compact_message(&help)))
        .unwrap_or_default();
    let location =
        if start.line == 0 { String::new() } else { format!(":{}:{}", start.line, start.column) };

    format!("{filename}{location}: {severity}{rule}: {message}{help}\n")
}

/// Collapse whitespace runs to single spaces and trim. Borrows when already compact,
/// which is the case for nearly every rule message.
fn compact_message(message: &str) -> Cow<'_, str> {
    if is_already_compact(message) {
        return Cow::Borrowed(message);
    }

    let mut compact = String::with_capacity(message.len());
    for word in message.split_whitespace() {
        if !compact.is_empty() {
            compact.push(' ');
        }
        compact.push_str(word);
    }
    Cow::Owned(compact)
}

/// True when every whitespace run is already a single space and there is none at either end.
fn is_already_compact(message: &str) -> bool {
    let mut after_space = true;
    for c in message.chars() {
        if c.is_whitespace() {
            if c != ' ' || after_space {
                return false;
            }
            after_space = true;
        } else {
            after_space = false;
        }
    }
    !after_space
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticReporter};
    use oxc_span::Span;

    use super::{AgentReporter, compact_message};

    // The borrowed fast path has to agree with the collapsing slow path on every input,
    // or messages silently change shape depending on which branch is taken.
    #[test]
    fn compact_message_fast_path_matches_slow_path() {
        fn collapse(message: &str) -> String {
            let mut compact = String::new();
            for word in message.split_whitespace() {
                if !compact.is_empty() {
                    compact.push(' ');
                }
                compact.push_str(word);
            }
            compact
        }

        for input in [
            "",
            " ",
            "   ",
            "a",
            "already compact",
            " leading",
            "trailing ",
            " both ",
            "double  space",
            "tab\tseparated",
            "newline\nseparated",
            "crlf\r\nseparated",
            "mixed \t\n runs",
            "non\u{a0}breaking",
            "trailing newline\n",
            "\nleading newline",
            "unicode \u{2192} \u{2716} ok",
        ] {
            assert_eq!(compact_message(input), collapse(input), "mismatch for {input:?}");
        }
    }

    #[test]
    fn compact_message_borrows_only_when_already_compact() {
        assert!(matches!(compact_message("already compact"), Cow::Borrowed(_)));
        assert!(matches!(compact_message("double  space"), Cow::Owned(_)));
        assert!(matches!(compact_message("trailing "), Cow::Owned(_)));
    }

    #[test]
    fn reporter_error() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::warn("error message")
            .with_error_code("eslint", "no-debugger")
            .with_help("help message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert_eq!(
            result.unwrap(),
            "file://test.ts:1:1: warning eslint(no-debugger): error message help: help message\n"
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

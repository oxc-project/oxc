use std::fmt::Write;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};
use rustc_hash::FxHashMap;

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct StylishOutputFormatter;

impl InternalFormatter for StylishOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(StylishReporter::default())
    }
}

#[derive(Default)]
struct StylishReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for StylishReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_stylish(&self.diagnostics))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_stylish(diagnostics: &[Error]) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    let mut grouped: FxHashMap<String, Vec<&Error>> = FxHashMap::default();
    let mut sorted = diagnostics.iter().collect::<Vec<_>>();

    sorted.sort_by_key(|diagnostic| Info::new(diagnostic).start.line);

    for diagnostic in sorted {
        let info = Info::new(diagnostic);
        grouped.entry(info.filename).or_default().push(diagnostic);
    }

    for diagnostics in grouped.values() {
        let diagnostic = diagnostics[0];
        let info = Info::new(diagnostic);
        let filename = info.filename;
        let filename = if let Some(path) =
            std::env::current_dir().ok().and_then(|d| d.join(&filename).canonicalize().ok())
        {
            path.display().to_string()
        } else {
            filename
        };
        let max_len_width = diagnostics
            .iter()
            .map(|diagnostic| {
                let start = Info::new(diagnostic).start;
                format!("{}:{}", start.line, start.column).len()
            })
            .max()
            .unwrap_or(0);

        writeln!(output, "\n\u{1b}[4m{filename}\u{1b}[0m").unwrap();

        for diagnostic in diagnostics {
            match diagnostic.severity() {
                Some(Severity::Error) => total_errors += 1,
                _ => total_warnings += 1,
            }

            let severity_str = if diagnostic.severity() == Some(Severity::Error) {
                "\u{1b}[31merror\u{1b}[0m"
            } else {
                "\u{1b}[33mwarning\u{1b}[0m"
            };

            let info = Info::new(diagnostic);
            let rule = diagnostic.code().map_or_else(String::new, |code| code.to_string());
            let position = format!("{}:{}", info.start.line, info.start.column);
            writeln!(
                output,
                "  \u{1b}[2m{position:max_len_width$}\u{1b}[0m  {severity_str}  {diagnostic}  \u{1b}[2m{rule}\u{1b}[0m"
            ).unwrap();
        }
    }

    let total = total_errors + total_warnings;
    if total > 0 {
        let summary_color = if total_errors > 0 { "\u{1b}[31m" } else { "\u{1b}[33m" };
        writeln!(
            output,
            "\n{summary_color}✖ {total} problem{} ({total_errors} error{}, {total_warnings} warning{})\u{1b}[0m",
            if total == 1 { "" } else { "s" },
            if total_errors == 1 { "" } else { "s" },
            if total_warnings == 1 { "" } else { "s" }
        ).unwrap();
    }

    output
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticResult};
    use oxc_span::Span;

    #[test]
    fn test_stylish_reporter() {
        let mut reporter = StylishReporter::default();

        let error = OxcDiagnostic::error("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "code"));

        let warning = OxcDiagnostic::warn("warning message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "code"));

        reporter.render_error(error);
        reporter.render_error(warning);

        let output = reporter.finish(&DiagnosticResult::default()).unwrap();

        assert!(output.contains("error message"), "Output should contain 'error message'");
        assert!(output.contains("warning message"), "Output should contain 'warning message'");
        assert!(output.contains("\u{2716}"), "Output should contain the ✖ character");
        assert!(output.contains("2 problems"), "Output should mention total problems");
        assert!(output.contains("1 error"), "Output should mention error count");
        assert!(output.contains("1 warning"), "Output should mention warning count");
    }
}

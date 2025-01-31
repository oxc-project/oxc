use oxc_diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
    Error, Severity,
};
use rustc_hash::FxHashMap;

use super::InternalFormatter;
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};

#[derive(Default)]
pub struct JUnitOutputFormatter;

impl InternalFormatter for JUnitOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(JUnitReporter::default())
    }
}

#[derive(Default)]
struct JUnitReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for JUnitReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_junit(&self.diagnostics))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_junit(diagnostics: &[Error]) -> String {
    let mut grouped: FxHashMap<String, Vec<&Error>> = FxHashMap::default();

    for diagnostic in diagnostics {
        let info = Info::new(diagnostic);
        grouped.entry(info.filename).or_default().push(diagnostic);
    }

    let mut report = Report::new("Oxlint");
    for diagnostics in grouped.values() {
        let diagnostic = diagnostics[0];
        let filename = Info::new(diagnostic).filename;

        let mut test_suite = TestSuite::new(filename);

        for diagnostic in diagnostics {
            let rule = diagnostic.code().map_or_else(String::new, |code| code.to_string());
            let Info { message, start, .. } = Info::new(diagnostic);

            let mut status = match diagnostic.severity() {
                Some(Severity::Error) => TestCaseStatus::non_success(NonSuccessKind::Error),
                _ => TestCaseStatus::non_success(NonSuccessKind::Failure),
            };
            status.set_message(message.clone());
            status.set_description(format!(
                "line {}, column {}, {}",
                start.line,
                start.column,
                message.clone()
            ));
            let test_case = TestCase::new(rule, status);
            test_suite.add_test_case(test_case);
        }
        report.add_test_suite(test_suite);
    }
    report.to_string().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_diagnostics::{reporter::DiagnosticResult, NamedSource, OxcDiagnostic};
    use oxc_span::Span;

    #[test]
    fn test_junit_reporter() {
        const EXPECTED_REPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Oxlint" tests="2" failures="1" errors="1">
    <testsuite name="file.js" tests="2" disabled="0" errors="1" failures="1">
        <testcase name="">
            <error message="error message">line 1, column 1, error message</error>
        </testcase>
        <testcase name="">
            <failure message="warning message">line 1, column 1, warning message</failure>
        </testcase>
    </testsuite>
</testsuites>
"#;
        let mut reporter = JUnitReporter::default();

        let error = OxcDiagnostic::error("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file.js", "let a = ;"));

        let warning = OxcDiagnostic::warn("warning message")
            .with_label(Span::new(0, 9))
            .with_source_code(NamedSource::new("file.js", "debugger;"));

        reporter.render_error(error);
        reporter.render_error(warning);

        let output = reporter.finish(&DiagnosticResult::default()).unwrap();
        assert_eq!(output.to_string(), EXPECTED_REPORT);
    }
}

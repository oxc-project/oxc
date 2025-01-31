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

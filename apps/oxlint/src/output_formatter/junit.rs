
use oxc_diagnostics::{reporter::{DiagnosticReporter, DiagnosticResult, Info}, Error};

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
    let mut report = Report::new("Oxlint");
    for diagnostic in diagnostics {
        let info = Info::new(diagnostic);
        let mut test_suite = TestSuite::new(info.filename);
        let mut status = TestCaseStatus::non_success(NonSuccessKind::Failure);
        status.set_message(info.message);
        let rule = diagnostic.code().map_or_else(String::new, |code| code.to_string());
        let test_case = TestCase::new(rule, status); 
        test_suite.add_test_case(test_case);

        report.add_test_suite(test_suite);
    }
    report.to_string().unwrap()
}
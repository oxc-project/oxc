use std::fmt::Write;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};
use rustc_hash::FxHashMap;

use super::{InternalFormatter, xml_utils::xml_escape};

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
    // `Info::new` scans the source to resolve the span, so build it exactly once per
    // diagnostic and group by index rather than re-deriving it inside the render loop.
    let infos: Vec<Info> = diagnostics.iter().map(Info::new).collect();

    let mut grouped: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    for (index, info) in infos.iter().enumerate() {
        grouped.entry(info.filename.as_str()).or_default().push(index);
    }

    let mut filenames: Vec<&str> = grouped.keys().copied().collect();
    filenames.sort_unstable();

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut test_suites = String::new();

    for (suite_index, filename) in filenames.iter().enumerate() {
        let indices = grouped.get(filename).expect("filename collected from map");
        let mut test_cases = String::new();
        let mut error = 0;
        let mut warning = 0;

        for &index in indices {
            let Info { message, start, rule_id, .. } = &infos[index];
            let rule = rule_id.as_deref().unwrap_or("");

            let severity = if diagnostics[index].severity() == Some(Severity::Error) {
                total_errors += 1;
                error += 1;
                "error"
            } else {
                total_warnings += 1;
                warning += 1;
                "failure"
            };
            let escaped_message = xml_escape(message);

            let _ = write!(
                test_cases,
                "\n        <testcase name=\"{rule}\">\n            <{severity} message=\"{escaped_message}\">line {}, column {}, {escaped_message}</{severity}>\n        </testcase>",
                start.line, start.column,
            );
        }

        if suite_index > 0 {
            test_suites.push('\n');
        }
        let _ = write!(
            test_suites,
            "    <testsuite name=\"{filename}\" tests=\"{}\" disabled=\"0\" errors=\"{error}\" failures=\"{warning}\">{test_cases}\n    </testsuite>",
            indices.len(),
        );
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"Oxlint\" tests=\"{}\" failures=\"{total_warnings}\" errors=\"{total_errors}\">\n{test_suites}\n</testsuites>\n",
        total_errors + total_warnings,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticResult};
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
        assert_eq!(output, EXPECTED_REPORT);
    }

    // The message is escaped once and interpolated into both the attribute and the element
    // body, so assert both. Also the only coverage of a non-empty rule name.
    #[test]
    fn test_junit_reporter_escapes_message_and_reports_rule() {
        const EXPECTED_REPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Oxlint" tests="1" failures="0" errors="1">
    <testsuite name="file.js" tests="1" disabled="0" errors="1" failures="0">
        <testcase name="eslint(no-debugger)">
            <error message="unexpected &apos;a &lt; b&apos; &amp; &quot;c&quot;">line 1, column 1, unexpected &apos;a &lt; b&apos; &amp; &quot;c&quot;</error>
        </testcase>
    </testsuite>
</testsuites>
"#;
        let mut reporter = JUnitReporter::default();

        reporter.render_error(
            OxcDiagnostic::error("unexpected 'a < b' & \"c\"")
                .with_error_code("eslint", "no-debugger")
                .with_label(Span::new(0, 8))
                .with_source_code(NamedSource::new("file.js", "debugger;")),
        );

        let output = reporter.finish(&DiagnosticResult::default()).unwrap();
        assert_eq!(output, EXPECTED_REPORT);
    }

    #[test]
    fn test_junit_reporter_multiple_files() {
        const EXPECTED_REPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Oxlint" tests="2" failures="1" errors="1">
    <testsuite name="a.js" tests="1" disabled="0" errors="1" failures="0">
        <testcase name="">
            <error message="error message a">line 1, column 1, error message a</error>
        </testcase>
    </testsuite>
    <testsuite name="b.js" tests="1" disabled="0" errors="0" failures="1">
        <testcase name="">
            <failure message="warning message b">line 1, column 1, warning message b</failure>
        </testcase>
    </testsuite>
</testsuites>
"#;

        let mut reporter = JUnitReporter::default();

        let error = OxcDiagnostic::error("error message a")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("a.js", "let a = ;"));

        let warning = OxcDiagnostic::warn("warning message b")
            .with_label(Span::new(0, 9))
            .with_source_code(NamedSource::new("b.js", "debugger;"));

        reporter.render_error(error);
        reporter.render_error(warning);

        let output = reporter.finish(&DiagnosticResult::default()).unwrap();
        assert_eq!(output, EXPECTED_REPORT);
    }
}

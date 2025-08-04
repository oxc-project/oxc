use std::hash::{DefaultHasher, Hash, Hasher};

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct GitlabOutputFormatter;

#[derive(Debug, serde::Serialize)]
struct GitlabErrorLocationLinesJson {
    begin: usize,
    end: usize,
}

#[derive(Debug, serde::Serialize)]
struct GitlabErrorLocationJson {
    path: String,
    lines: GitlabErrorLocationLinesJson,
}

#[derive(Debug, serde::Serialize)]
struct GitlabErrorJson {
    description: String,
    check_name: String,
    fingerprint: String,
    severity: String,
    location: GitlabErrorLocationJson,
}

impl InternalFormatter for GitlabOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GitlabReporter::default())
    }
}

/// Renders reports as a Gitlab Code Quality Report
///
/// <https://docs.gitlab.com/ci/testing/code_quality/#code-quality-report-format>
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
#[derive(Default)]
struct GitlabReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for GitlabReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_gitlab(&mut self.diagnostics))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_gitlab(diagnostics: &mut Vec<Error>) -> String {
    let errors = diagnostics.drain(..).map(|error| {
        let Info { start, end, filename, message, severity, rule_id } = Info::new(&error);
        let severity = match severity {
            Severity::Error => "critical".to_string(),
            Severity::Warning => "major".to_string(),
            Severity::Advice => "minor".to_string(),
        };

        let fingerprint = {
            let mut hasher = DefaultHasher::new();
            start.line.hash(&mut hasher);
            end.line.hash(&mut hasher);
            filename.hash(&mut hasher);
            message.hash(&mut hasher);
            severity.hash(&mut hasher);

            format!("{:x}", hasher.finish())
        };

        GitlabErrorJson {
            description: message,
            check_name: rule_id.unwrap_or_default(),
            location: GitlabErrorLocationJson {
                path: filename,
                lines: GitlabErrorLocationLinesJson { begin: start.line, end: end.line },
            },
            fingerprint,
            severity,
        }
    });

    serde_json::to_string_pretty(&errors.collect::<Vec<_>>()).expect("Failed to serialize")
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{
        NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::GitlabReporter;

    #[test]
    fn reporter() {
        let mut reporter = GitlabReporter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let first_result = reporter.render_error(error);

        // reporter keeps it in memory
        assert!(first_result.is_none());

        // reporter gives results when finishing
        let second_result = reporter.finish(&DiagnosticResult::default());

        assert!(second_result.is_some());
        let json: serde_json::Value = serde_json::from_str(&second_result.unwrap()).unwrap();
        let array = json.as_array().unwrap();
        assert_eq!(array.len(), 1);
        let value = array[0].as_object().unwrap();
        assert_eq!(value.keys().len(), 5);
        assert_eq!(value["description"], "error message");
        assert_eq!(value["check_name"], "");
        assert!(value["fingerprint"].is_string()); // value is different on different architectures
        assert_eq!(value["severity"], "major");
        let location = value["location"].as_object().unwrap();
        assert_eq!(location["path"], "file://test.ts");
        let lines = location["lines"].as_object().unwrap();
        assert_eq!(lines["begin"], 1);
        assert_eq!(lines["end"], 1);
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use cow_utils::CowUtils;

use serde::Serialize;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct BitbucketOutputFormatter;

/// Severity levels accepted by the Bitbucket Code Insights Annotations API.
///
/// <https://developer.atlassian.com/cloud/bitbucket/rest/api-group-reports/#api-group-reports>
#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[expect(dead_code)]
enum BitbucketSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Annotation types accepted by the Bitbucket Code Insights Annotations API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[expect(dead_code)]
enum BitbucketAnnotationType {
    Bug,
    CodeSmell,
    Vulnerability,
}

/// A single Bitbucket Code Insights annotation entry.
///
/// Matches the schema required by the bulk create/update annotations endpoint:
/// `POST /repositories/{workspace}/{repo_slug}/commit/{commit}/reports/{reportId}/annotations`
#[derive(Debug, Serialize)]
struct BitbucketAnnotationJson {
    external_id: String,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    annotation_type: BitbucketAnnotationType,
    severity: BitbucketSeverity,
    path: String,
    line: usize,
}

impl InternalFormatter for BitbucketOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(BitbucketReporter::default())
    }
}

/// Find the git repository root by walking up from the current directory.
/// Returns `None` if no `.git` directory is found.
fn find_git_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_git_root_from(&cwd)
}

/// Find the git repository root by walking up from the given path.
fn find_git_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Get the path prefix from CWD to the git repository root.
/// This prefix should be prepended to CWD-relative paths to make them repo-relative.
///
/// For example, if git root is `/repo` and CWD is `/repo/packages/foo`,
/// this returns `Some("packages/foo")`.
fn get_repo_path_prefix() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let git_root = find_git_root()?;

    let relative = cwd.strip_prefix(&git_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }

    Some(relative.to_path_buf())
}

/// Renders reports as a Bitbucket Code Insights annotations array.
///
/// The output JSON array can be sent directly to the Bitbucket bulk annotations endpoint:
/// `POST /repositories/{workspace}/{repo_slug}/commit/{commit}/reports/{reportId}/annotations`
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
struct BitbucketReporter {
    diagnostics: Vec<Error>,
    /// Path prefix to prepend to CWD-relative paths to make them repo-relative.
    /// `None` if CWD is the git root or if we're not in a git repository.
    repo_path_prefix: Option<PathBuf>,
}

impl BitbucketReporter {
    fn default() -> Self {
        Self { diagnostics: Vec::new(), repo_path_prefix: get_repo_path_prefix() }
    }
}

impl DiagnosticReporter for BitbucketReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_bitbucket(&mut self.diagnostics, self.repo_path_prefix.as_deref()))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_bitbucket(diagnostics: &mut Vec<Error>, repo_path_prefix: Option<&Path>) -> String {
    let annotations = diagnostics.drain(..).map(|error| {
        let Info { start, filename, message, severity, rule_id, .. } = Info::new(&error);

        let (annotation_type, bitbucket_severity) = match severity {
            Severity::Error => (BitbucketAnnotationType::Bug, BitbucketSeverity::High),
            Severity::Warning => (BitbucketAnnotationType::CodeSmell, BitbucketSeverity::Medium),
            Severity::Advice => (BitbucketAnnotationType::CodeSmell, BitbucketSeverity::Low),
        };

        let external_id = {
            let mut hasher = DefaultHasher::new();
            start.line.hash(&mut hasher);
            filename.hash(&mut hasher);
            message.hash(&mut hasher);
            rule_id.hash(&mut hasher);
            format!("oxlint-{:x}", hasher.finish())
        };

        let path = match repo_path_prefix {
            Some(prefix) => {
                // Normalize path separators to forward slashes on Windows.
                #[cfg(windows)]
                {
                    let combined = prefix.join(&filename);
                    combined.to_string_lossy().cow_replace('\\', "/").into_owned()
                }
                #[cfg(not(windows))]
                {
                    prefix.join(&filename).to_string_lossy().to_string()
                }
            }
            None => filename,
        };

        BitbucketAnnotationJson {
            external_id,
            summary: message,
            details: rule_id,
            annotation_type,
            severity: bitbucket_severity,
            path,
            line: start.line,
        }
    });

    serde_json::to_string_pretty(&annotations.collect::<Vec<_>>()).expect("Failed to serialize")
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use oxc_diagnostics::{
        Error, NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::{BitbucketReporter, find_git_root_from, format_bitbucket};

    #[test]
    fn reporter() {
        let mut reporter = BitbucketReporter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("test.ts", "debugger;"));

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
        assert!(value["external_id"].as_str().unwrap().starts_with("oxlint-"));
        assert_eq!(value["summary"], "error message");
        assert_eq!(value["annotation_type"], "CODE_SMELL");
        assert_eq!(value["severity"], "MEDIUM");
        let location_path = value["path"].as_str().unwrap();
        assert!(
            location_path.ends_with("test.ts"),
            "path '{location_path}' should end with test.ts"
        );
        assert_eq!(value["line"], 1);
    }

    #[test]
    fn reporter_with_error_severity() {
        let mut reporter = BitbucketReporter { diagnostics: Vec::new(), repo_path_prefix: None };

        let error = OxcDiagnostic::error("critical error")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("example.js", "eval('x');"));

        reporter.render_error(error);
        let result = reporter.finish(&DiagnosticResult::default()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let value = &json[0];
        assert_eq!(value["annotation_type"], "BUG");
        assert_eq!(value["severity"], "HIGH");
        assert_eq!(value["path"], "example.js");
    }

    #[test]
    fn find_git_root_from_current_dir() {
        let cwd = std::env::current_dir().unwrap();
        let git_root = find_git_root_from(&cwd);
        assert!(git_root.is_some());
        assert!(git_root.unwrap().join(".git").exists());
    }

    #[test]
    fn find_git_root_from_nonexistent() {
        let path = PathBuf::from("/");
        let git_root = find_git_root_from(&path);
        assert!(git_root.is_none() || *git_root.unwrap() == *"/");
    }

    #[test]
    fn format_bitbucket_with_prefix() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        let result = format_bitbucket(&mut diagnostics, Some(Path::new("packages/foo")));
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["path"].as_str().unwrap();
        assert_eq!(path, "packages/foo/example.js");
    }

    #[test]
    fn format_bitbucket_without_prefix() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        let result = format_bitbucket(&mut diagnostics, None);
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["path"].as_str().unwrap();
        assert_eq!(path, "example.js");
    }

    #[test]
    fn format_bitbucket_external_id_uniqueness() {
        // Two identical errors should produce the same external_id (deterministic hashing)
        let error1 = OxcDiagnostic::warn("duplicate")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("file.js", "const x = 1;"));
        let error2 = OxcDiagnostic::warn("duplicate")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("file.js", "const x = 1;"));

        let mut d1: Vec<Error> = vec![error1];
        let mut d2: Vec<Error> = vec![error2];

        let r1 = format_bitbucket(&mut d1, None);
        let r2 = format_bitbucket(&mut d2, None);

        let j1: serde_json::Value = serde_json::from_str(&r1).unwrap();
        let j2: serde_json::Value = serde_json::from_str(&r2).unwrap();

        assert_eq!(j1[0]["external_id"], j2[0]["external_id"]);
    }

    #[cfg(windows)]
    #[test]
    fn format_bitbucket_windows_normalization() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        // Windows-style prefix with backslashes should be normalized to forward slashes
        let result = format_bitbucket(&mut diagnostics, Some(Path::new(r"packages\foo")));
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["path"].as_str().unwrap();
        assert_eq!(path, "packages/foo/example.js");
    }
}

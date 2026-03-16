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
pub struct GitlabOutputFormatter;

#[derive(Debug, Serialize)]
struct GitlabErrorLocationLinesJson {
    begin: usize,
    end: usize,
}

#[derive(Debug, Serialize)]
struct GitlabErrorLocationJson {
    path: String,
    lines: GitlabErrorLocationLinesJson,
}

#[derive(Debug, Serialize)]
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

    // Get the relative path from git root to CWD
    let relative = cwd.strip_prefix(&git_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }

    Some(relative.to_path_buf())
}

/// Renders reports as a Gitlab Code Quality Report
///
/// <https://docs.gitlab.com/ci/testing/code_quality/#code-quality-report-format>
///
/// Note that, due to syntactic restrictions of JSON arrays, this reporter waits until all
/// diagnostics have been reported before writing them to the output stream.
struct GitlabReporter {
    diagnostics: Vec<Error>,
    /// Path prefix to prepend to CWD-relative paths to make them repo-relative.
    /// `None` if CWD is the git root or if we're not in a git repository.
    repo_path_prefix: Option<PathBuf>,
}

impl GitlabReporter {
    fn default() -> Self {
        Self { diagnostics: Vec::new(), repo_path_prefix: get_repo_path_prefix() }
    }
}

impl DiagnosticReporter for GitlabReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_gitlab(&mut self.diagnostics, self.repo_path_prefix.as_deref()))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_gitlab(diagnostics: &mut Vec<Error>, repo_path_prefix: Option<&Path>) -> String {
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
                // GitLab expects file paths to be relative to the repository
                // root, so adjust accordingly.
                path: match repo_path_prefix {
                    Some(prefix) => {
                        // only do the path swap on Windows
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
                },
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
    use std::path::{Path, PathBuf};

    use oxc_diagnostics::{
        Error, NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::{GitlabReporter, find_git_root_from, format_gitlab};

    #[test]
    fn reporter() {
        let mut reporter = GitlabReporter::default();

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
        assert_eq!(value.keys().len(), 5);
        assert_eq!(value["description"], "error message");
        assert_eq!(value["check_name"], "");
        assert!(value["fingerprint"].is_string()); // value is different on different architectures
        assert_eq!(value["severity"], "major");
        let location = value["location"].as_object().unwrap();
        assert_eq!(location["path"], "apps/oxlint/test.ts");
        let lines = location["lines"].as_object().unwrap();
        assert_eq!(lines["begin"], 1);
        assert_eq!(lines["end"], 1);
    }

    #[test]
    fn find_git_root_from_current_dir() {
        // This test runs from within the oxc repo, so we should find a git root
        let cwd = std::env::current_dir().unwrap();
        let git_root = find_git_root_from(&cwd);
        assert!(git_root.is_some());
        assert!(git_root.unwrap().join(".git").exists());
    }

    #[test]
    fn find_git_root_from_nonexistent() {
        // A path that doesn't exist or has no git repo
        let path = PathBuf::from("/");
        let git_root = find_git_root_from(&path);
        // Root directory typically doesn't have a .git folder
        assert!(git_root.is_none() || *git_root.unwrap() == *"/");
    }

    #[test]
    fn format_gitlab_with_prefix() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        // Test with a prefix
        let result = format_gitlab(&mut diagnostics, Some(Path::new("packages/foo")));
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["location"]["path"].as_str().unwrap();
        assert_eq!(path, "packages/foo/example.js");
    }

    #[test]
    fn format_gitlab_without_prefix() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        // Test without a prefix (CWD is at git root)
        let result = format_gitlab(&mut diagnostics, None);
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["location"]["path"].as_str().unwrap();
        assert_eq!(path, "example.js");
    }

    #[cfg(windows)]
    #[test]
    fn format_gitlab_windows_normalization() {
        let error = OxcDiagnostic::warn("test error")
            .with_label(Span::new(0, 5))
            .with_source_code(NamedSource::new("example.js", "const x = 1;"));

        let mut diagnostics: Vec<Error> = vec![error];

        // Windows-style prefix with backslashes should be normalized to forward slashes
        let result = format_gitlab(&mut diagnostics, Some(Path::new(r"packages\foo")));
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let path = json[0]["location"]["path"].as_str().unwrap();
        assert_eq!(path, "packages/foo/example.js");
    }
}

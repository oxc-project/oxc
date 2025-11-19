//! Coverage Reporter - Unified output and snapshot generation
//!
//! This module handles all coverage reporting:
//! - Console output with statistics
//! - Snapshot file generation for conformance tracking

use std::io::{Write, stdout};
use std::path::Path;

use oxc_tasks_common::Snapshot;

use crate::suite::{ExecutedTest, TestResult};
use crate::{snap_root, workspace_root};

/// Computed statistics for a test suite run
pub struct CoverageStats {
    pub all_positives: usize,
    pub parsed_positives: usize,
    pub passed_positives: usize,
    pub all_negatives: usize,
    pub passed_negatives: usize,
}

impl CoverageStats {
    /// Compute statistics from test results
    pub fn from_results(results: &[ExecutedTest]) -> Self {
        let (negatives, positives): (Vec<_>, Vec<_>) =
            results.iter().partition(|t| t.should_fail());

        let all_positives = positives.len();
        let parsed_positives = positives.iter().filter(|t| t.test_parsed()).count();
        let passed_positives = positives.iter().filter(|t| t.test_passed()).count();

        let all_negatives = negatives.len();
        let passed_negatives = negatives.iter().filter(|t| t.test_passed()).count();

        Self { all_positives, parsed_positives, passed_positives, all_negatives, passed_negatives }
    }

    /// Calculate parsed percentage
    #[expect(clippy::cast_precision_loss)]
    fn parsed_percent(&self) -> f64 {
        (self.parsed_positives as f64 / self.all_positives as f64) * 100.0
    }

    /// Calculate positive passed percentage
    #[expect(clippy::cast_precision_loss)]
    fn positive_percent(&self) -> f64 {
        (self.passed_positives as f64 / self.all_positives as f64) * 100.0
    }

    /// Calculate negative passed percentage
    #[expect(clippy::cast_precision_loss)]
    fn negative_percent(&self) -> f64 {
        (self.passed_negatives as f64 / self.all_negatives as f64) * 100.0
    }
}

/// Configuration for the reporter
pub struct Reporter {
    /// Whether to print detailed failure information
    pub detail: bool,
    /// Whether to save snapshots (disabled when filtering)
    pub save_snapshots: bool,
}

impl Reporter {
    /// Create a new reporter with the given configuration
    pub fn new(detail: bool, save_snapshots: bool) -> Self {
        Self { detail, save_snapshots }
    }

    /// Print and optionally save coverage results
    pub fn report(&self, name: &str, results: &[ExecutedTest], test_root: &Path) {
        let stats = CoverageStats::from_results(results);

        self.print_to_stdout(name, results, &stats);

        if self.save_snapshots {
            save_snapshot(name, results, test_root, &stats);
        }
    }

    /// Print coverage summary and failures to stdout
    fn print_to_stdout(&self, name: &str, results: &[ExecutedTest], stats: &CoverageStats) {
        let mut out = stdout();

        writeln!(out, "{name} Summary:").unwrap();
        writeln!(
            out,
            "AST Parsed     : {}/{} ({:.2}%)",
            stats.parsed_positives,
            stats.all_positives,
            stats.parsed_percent()
        )
        .unwrap();
        writeln!(
            out,
            "Positive Passed: {}/{} ({:.2}%)",
            stats.passed_positives,
            stats.all_positives,
            stats.positive_percent()
        )
        .unwrap();

        if stats.all_negatives > 0 {
            writeln!(
                out,
                "Negative Passed: {}/{} ({:.2}%)",
                stats.passed_negatives,
                stats.all_negatives,
                stats.negative_percent()
            )
            .unwrap();
        }

        // Print failed tests if detail mode
        if self.detail {
            let (negatives, positives): (Vec<_>, Vec<_>) =
                results.iter().partition(|t| t.should_fail());

            let mut failed_negatives: Vec<_> =
                negatives.iter().filter(|t| !t.test_passed()).collect();
            failed_negatives.sort_by_key(|t| t.path());

            let mut failed_positives: Vec<_> =
                positives.iter().filter(|t| !t.test_passed()).collect();
            failed_positives.sort_by_key(|t| t.path());

            for test in failed_negatives.iter().chain(failed_positives.iter()) {
                if let Some(error) = test.error_message() {
                    writeln!(out, "{}\n{}", test.path().display(), error).unwrap();
                }
            }
        }

        out.flush().unwrap();
    }
}

/// Save snapshot file for results
///
/// Snapshot format:
/// 1. Write summary stats
/// 2. Write failed test details (failed negatives + failed positives)
/// 3. Append CorrectError contents (raw error strings)
fn save_snapshot(name: &str, results: &[ExecutedTest], test_root: &Path, stats: &CoverageStats) {
    use std::io::Write;

    let snapshot_path = workspace_root().join(test_root);
    let show_commit = !snapshot_path.to_string_lossy().contains("misc");
    let snapshot = Snapshot::new(&snapshot_path, show_commit);

    let mut out: Vec<u8> = vec![];

    // 1. Write coverage summary
    writeln!(out, "{name} Summary:").unwrap();
    writeln!(
        out,
        "AST Parsed     : {}/{} ({:.2}%)",
        stats.parsed_positives,
        stats.all_positives,
        stats.parsed_percent()
    )
    .unwrap();
    writeln!(
        out,
        "Positive Passed: {}/{} ({:.2}%)",
        stats.passed_positives,
        stats.all_positives,
        stats.positive_percent()
    )
    .unwrap();

    if stats.all_negatives > 0 {
        writeln!(
            out,
            "Negative Passed: {}/{} ({:.2}%)",
            stats.passed_negatives,
            stats.all_negatives,
            stats.negative_percent()
        )
        .unwrap();
    }

    // 2. Write failed test details
    let (negatives, positives): (Vec<_>, Vec<_>) = results.iter().partition(|t| t.should_fail());

    let mut failed_negatives: Vec<_> = negatives.iter().filter(|t| !t.test_passed()).collect();
    failed_negatives.sort_by_key(|t| t.path());

    let mut failed_positives: Vec<_> = positives.iter().filter(|t| !t.test_passed()).collect();
    failed_positives.sort_by_key(|t| t.path());

    for test in failed_negatives.iter().chain(failed_positives.iter()) {
        let path = format!("tasks/coverage/{}", test.path().display());
        match test.test_result() {
            TestResult::IncorrectlyPassed => {
                // Negative test that should have failed but passed
                writeln!(out, "Expect Syntax Error: {path}\n").unwrap();
            }
            TestResult::ParseError(error, _) => {
                // Positive test that failed to parse
                // If error already contains the path (e.g., "semantic Error: {path}"),
                // don't add another prefix - these errors already have proper formatting
                if error.contains("Error: tasks/coverage/")
                    || error.contains("Error: ")
                        && error.contains(&test.path().to_string_lossy().to_string())
                {
                    out.extend(error.as_bytes());
                    // These errors already end with \n, add one more for blank line
                    out.push(b'\n');
                } else {
                    writeln!(out, "Expect to Parse: {path}").unwrap();
                    out.extend(error.as_bytes());
                    // Errors with graphical boxes already end with \n
                    // Simple errors don't, so add one \n to end the line
                    // Main's format has no blank lines between simple ParseError entries
                    if error.ends_with('\n') {
                        // Box error: already has \n, just add one more for blank line
                        out.push(b'\n');
                    } else {
                        // Simple error: add \n for end of line only (no blank line)
                        out.push(b'\n');
                    }
                }
            }
            TestResult::Mismatch(case, _, _) => {
                // Mismatch in output (codegen, transformer, etc.)
                writeln!(out, "{case}: {path}\n").unwrap();
            }
            TestResult::GenericError(case, error) => {
                // Generic tool error
                writeln!(out, "{case} Error: {path}").unwrap();
                writeln!(out, "{error}\n").unwrap();
            }
            _ => {}
        }
    }

    // 3. Append CorrectError contents
    // These are negative tests that correctly produced errors
    let mut correct_errors: Vec<_> = results
        .iter()
        .filter(|t| matches!(t.test_result(), TestResult::CorrectError(_, _)))
        .collect();
    correct_errors.sort_by_key(|t| t.path());

    for test in &correct_errors {
        if let TestResult::CorrectError(error, _) = test.test_result() {
            out.extend(error.as_bytes());
        }
    }

    let path = snap_root().join(format!("{}.snap", name.to_lowercase()));
    let content = String::from_utf8(out).unwrap();
    snapshot.save(&path, &content);
}

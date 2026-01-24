mod constants;
mod diagnostics_code_collector;
mod meta;
mod transpile_runner;

use std::path::{Path, PathBuf};

use oxc::{diagnostics::OxcDiagnostic, span::Span};

use self::meta::{CompilerSettings, TestCaseContent, TestUnitData};
pub use self::transpile_runner::{TranspileRunner, TypeScriptTranspileCase};
use crate::suite::{Case, Suite, TestResult};
pub use diagnostics_code_collector::save_reviewed_tsc_diagnostics_codes;

const TESTS_ROOT: &str = "typescript/tests";

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(TESTS_ROOT).join("cases"), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TypeScriptSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        // stack overflows in compiler tests
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));
        let unsupported_tests =
            constants::NOT_SUPPORTED_TEST_PATHS.iter().any(|p| path.to_string_lossy().contains(p));
        !supported_paths || unsupported_tests
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct TypeScriptCase {
    path: PathBuf,
    pub code: String,
    pub units: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    error_codes: Vec<String>,
    pub result: TestResult,
}

impl TypeScriptCase {
    /// Simple check for usage such as `semantic`.
    /// `should_fail()` will return `true` only if there are still error codes remaining
    /// after filtering out the not-supported ones.
    pub fn should_fail_with_any_error_codes(&self) -> bool {
        !self.error_codes.is_empty()
    }

    /// Check if a diagnostic error is suppressed by a `@ts-ignore` or `@ts-expect-error`
    /// comment on the preceding line.
    fn is_error_suppressed_by_ts_ignore(
        error: &OxcDiagnostic,
        source_text: &str,
        ts_ignore_spans: &[Span],
    ) -> bool {
        // Check if this error message is suppressible
        let error_message = error.to_string();
        if !constants::TS_IGNORE_SUPPRESSIBLE_ERRORS.contains(error_message.as_str()) {
            return false;
        }

        // Get the error's byte offset from the first label
        let Some(labels) = &error.labels else {
            return false;
        };
        let Some(first_label) = labels.first() else {
            return false;
        };
        let error_offset = first_label.offset();

        // Check if any ts-ignore span covers the line before this error
        for ts_ignore_span in ts_ignore_spans {
            let after_comment = &source_text[ts_ignore_span.end as usize..];

            // Find the first newline (end of the comment line)
            let Some(first_newline_pos) = after_comment.find('\n') else {
                continue;
            };

            // The next line starts after the first newline
            let next_line_start = ts_ignore_span.end as usize + first_newline_pos + 1;

            // Find the end of the next line (second newline or end of string)
            let next_line_end = source_text[next_line_start..]
                .find('\n')
                .map_or(source_text.len(), |pos| next_line_start + pos);

            // Check if the error offset falls within the next line
            if error_offset >= next_line_start && error_offset < next_line_end {
                return true;
            }
        }

        false
    }
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, error_codes } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, error_codes, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        // If there are still error codes to be supported, it should fail
        self.error_codes
            .iter()
            .any(|code| !constants::NOT_SUPPORTED_ERROR_CODES.contains(code.as_str()))
    }

    fn always_strict(&self) -> bool {
        self.settings.always_strict
    }

    fn run(&mut self) {
        let result = self
            .units
            .iter()
            .map(|unit| {
                let parse_result = self.parse(&unit.content, unit.source_type);
                // If parsing failed, check if all errors are suppressed by @ts-ignore
                match parse_result {
                    Err((errors, error_output, panicked)) => {
                        // Filter out errors that are suppressed by ts-ignore
                        let has_unsuppressed_errors = errors.iter().any(|error| {
                            !Self::is_error_suppressed_by_ts_ignore(
                                error,
                                &unit.content,
                                &unit.ts_ignore_spans,
                            )
                        });

                        if has_unsuppressed_errors {
                            Err((errors, error_output, panicked))
                        } else {
                            Ok(())
                        }
                    }
                    Ok(()) => Ok(()),
                }
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()));
        self.result = self.evaluate_result(result);
    }
}

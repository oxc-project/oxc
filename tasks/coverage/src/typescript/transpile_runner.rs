//! <https://github.com/microsoft/TypeScript/blob/v5.6.3/src/testRunner/transpileRunner.ts>

use std::path::{Path, PathBuf};

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CommentOptions},
    diagnostics::OxcDiagnostic,
    isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions},
    parser::Parser,
    span::SourceType,
};

use super::{
    TESTS_ROOT,
    meta::{Baseline, BaselineFile},
};
use crate::{
    suite::{
        ErrorKind, ExecutionOutput, ExecutionResult, LoadedTest, ParsedTest, ResultValidator,
        TestFilter, TestMetadata, TestResult, TestRunner,
    },
    workspace_root,
};

/// Custom validator for transpiler tests
///
/// This validator handles the special case where diagnostic mismatches
/// should still be counted as "passed" but output to the snapshot.
/// This matches the legacy behavior where `CorrectError` was returned
/// for diagnostic mismatches.
pub struct TranspileValidator;

impl ResultValidator for TranspileValidator {
    fn validate(&self, test: &LoadedTest, result: ExecutionResult) -> TestResult {
        match result.error_kind {
            ErrorKind::None => {
                if test.should_fail {
                    TestResult::IncorrectlyPassed
                } else {
                    TestResult::Passed
                }
            }
            ErrorKind::Errors(errors) => {
                let error_msg = errors.join("\n");
                // For transpiler, errors are diagnostic snapshots that should be
                // counted as "correct" (passed with output) regardless of should_fail
                TestResult::CorrectError(error_msg, result.panicked)
            }
            ErrorKind::Mismatch { case, actual, expected } => {
                TestResult::Mismatch(case, actual, expected)
            }
            ErrorKind::Generic { case, error } => TestResult::GenericError(case, error),
        }
    }
}

// Helper functions extracted from removed TypeScriptTranspileCase impl
fn change_extension(name: &str) -> String {
    Path::new(name).with_extension("").with_extension("d.ts").to_str().unwrap().to_string()
}

fn transpile(path: &Path, source_text: &str) -> (String, Vec<OxcDiagnostic>) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&ret.program);
    let printed = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { jsdoc: true, ..CommentOptions::disabled() },
            ..CodegenOptions::default()
        })
        .build(&ret.program)
        .code;
    (printed, ret.errors)
}

/// Transpile test filter - only runs tests with @declaration: true
pub struct TranspileFilter;

impl TranspileFilter {
    pub const fn new() -> Self {
        Self
    }
}

impl TestFilter for TranspileFilter {
    fn skip_path(&self, path: &Path) -> bool {
        // Check for unsupported test paths (from TypeScript constants)
        super::constants::NOT_SUPPORTED_TEST_PATHS
            .iter()
            .any(|p| path.to_string_lossy().contains(p))
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        // Only run tests with @declaration: true
        match &test.metadata {
            TestMetadata::TypeScript { settings, .. } => !settings.declaration,
            _ => true,
        }
    }
}

/// Transpile - Implements the generalized TestRunner trait
pub struct TranspileRunner;

impl TestRunner for TranspileRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        let TestMetadata::TypeScript { units, settings, .. } = &test.metadata else {
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![
                    "Not a TypeScript test".to_string(),
                ]),
                panicked: false,
            });
        };

        // Only run if @declaration: true
        if !settings.declaration {
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            });
        }

        // Get expected baseline file
        // Try multiple prefix patterns to handle path format variations
        let path = test
            .id
            .strip_prefix("typescript/tests/cases/transpile/")
            .or_else(|| test.id.strip_prefix("typescript/tests/cases/transpile"))
            .or_else(|| test.id.strip_prefix("/typescript/tests/cases/transpile/"))
            .or_else(|| test.id.strip_prefix("/typescript/tests/cases/transpile"));
        let Some(path) = path else {
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![format!(
                    "Invalid path: {}",
                    test.id
                )]),
                panicked: false,
            });
        };
        // Remove any leading slash
        let path = path.strip_prefix('/').unwrap_or(path);
        let filename = change_extension(path);
        let baseline_path =
            workspace_root().join(TESTS_ROOT).join("baselines/reference/transpile").join(filename);
        let expected = BaselineFile::parse(&baseline_path);

        // Generate baseline from test units
        let baseline = Self::run_transpile(&PathBuf::from(&test.id), units);

        // Compare
        let expected_text = expected.print();
        let baseline_text = baseline.print();

        if expected.files.len() != baseline.files.len() {
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Mismatch {
                    case: "Mismatch",
                    actual: baseline_text,
                    expected: expected_text,
                },
                panicked: false,
            });
        }

        for (base, expected) in baseline.files.iter().zip(expected.files) {
            if expected.original_diagnostic.is_empty() {
                if base.oxc_printed != expected.oxc_printed {
                    return Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Mismatch {
                            case: "Mismatch",
                            actual: base.oxc_printed.clone(),
                            expected: expected.oxc_printed,
                        },
                        panicked: false,
                    });
                }
            } else {
                let matched = base.oxc_diagnostics.iter().zip(&expected.original_diagnostic).all(
                    |(base_diagnostic, expected_diagnostic)| {
                        expected_diagnostic.contains(&base_diagnostic.to_string())
                    },
                );
                if !matched {
                    // Diagnostic mismatch - output snapshot as CorrectError (passes but shows output)
                    let snapshot = format!("\n#### {} ####\n{}", test.id, baseline.snapshot());
                    return Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Errors(vec![snapshot]),
                        panicked: false,
                    });
                }
                // Diagnostics matched - test passes with no output
            }
        }

        Some(ExecutionResult {
            output: ExecutionOutput::None,
            error_kind: crate::suite::ErrorKind::None,
            panicked: false,
        })
    }

    fn name(&self) -> &'static str {
        "transpile"
    }
}

impl TranspileRunner {
    fn run_transpile(path: &Path, units: &[crate::suite::TypeScriptUnit]) -> BaselineFile {
        let mut files = vec![];

        // Add original files
        for unit in units {
            let mut baseline = Baseline {
                name: unit.name.clone(),
                original: unit.content.clone(),
                ..Baseline::default()
            };
            baseline.print_oxc();
            files.push(baseline);
        }

        // Add transpiled .d.ts files
        for unit in units {
            let (source_text, errors) = transpile(path, &unit.content);
            let baseline = Baseline {
                name: change_extension(&unit.name),
                original: unit.content.clone(),
                original_diagnostic: Vec::default(),
                oxc_printed: source_text,
                oxc_diagnostics: errors,
            };
            files.push(baseline);
        }

        BaselineFile { files }
    }
}

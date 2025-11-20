use oxc::span::SourceType;

use crate::{
    Driver,
    suite::{ExecutionOutput, ExecutionResult, LoadedTest, TestResult, TestRunner},
};

/// Codegen Runner - Implements the TestRunner trait
pub struct CodegenRunner;

impl TestRunner for CodegenRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        // Handle TypeScript tests with multiple compilation units
        if let Some(units) = test.typescript_units() {
            for (content, source_type) in units {
                let result = Self::run_idempotency(&content, source_type);
                if result != TestResult::Passed {
                    return Some(result.into());
                }
            }
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            });
        }

        // Normal single-file tests
        let result = Self::run_idempotency(&test.code, test.source_type);
        Some(result.into())
    }

    fn name(&self) -> &'static str {
        "codegen"
    }
}

impl CodegenRunner {
    fn run_idempotency(source_text: &str, source_type: SourceType) -> TestResult {
        // Run normal idempotency
        let result = Driver { codegen: true, ..Driver::default() }.idempotency(
            "Normal",
            source_text,
            source_type,
        );
        if result != TestResult::Passed {
            return result;
        }

        // Run minified idempotency
        Driver { codegen: true, remove_whitespace: true, ..Driver::default() }.idempotency(
            "Minify",
            source_text,
            source_type,
        )
    }
}

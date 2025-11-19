use oxc::{
    span::SourceType,
    transformer::{JsxRuntime, TransformOptions},
};

use crate::{
    driver::Driver,
    suite::{ExecutionOutput, ExecutionResult, LoadedTest, TestResult, TestRunner},
    tools::get_default_transformer_options,
};

/// Transformer Runner - Implements the TestRunner trait
pub struct TransformerRunner;

impl TestRunner for TransformerRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        // Handle TypeScript tests with multiple compilation units
        if let Some((units, settings)) = test.typescript_units_with_settings() {
            for (content, source_type) in units {
                // Build options with JSX settings
                let mut options = get_default_transformer_options(None);
                let mut source_type = source_type;

                // Handle @jsx: react - match babel behavior
                if settings.jsx.last().is_some_and(|jsx| jsx == "react") {
                    source_type = source_type.with_module(true);
                    options.jsx.runtime = JsxRuntime::Classic;
                }

                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::run_idempotency(&content, source_type, &test.id, Some(options))
                }));
                if let Ok(result) = result {
                    if result != TestResult::Passed {
                        return Some(result.into());
                    }
                } else {
                    return Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Errors(vec![format!(
                            "Panic in test: {}",
                            test.id
                        )]),
                        panicked: true,
                    });
                }
            }
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            });
        }

        // Normal single-file tests
        let result = Self::run_idempotency(&test.code, test.source_type, &test.id, None);
        Some(result.into())
    }

    fn name(&self) -> &'static str {
        "transformer"
    }
}

impl TransformerRunner {
    fn run_idempotency(
        source_text: &str,
        source_type: SourceType,
        path_id: &str,
        options: Option<TransformOptions>,
    ) -> TestResult {
        use std::path::PathBuf;

        let mut driver = Driver {
            path: PathBuf::from(path_id),
            transform: Some(options.unwrap_or_else(|| get_default_transformer_options(None))),
            codegen: true,
            ..Driver::default()
        };
        let transformed1 = {
            driver.run(source_text, source_type);
            driver.printed.clone()
        };
        // Second pass with only JavaScript syntax
        let transformed2 = {
            driver.run(&transformed1, SourceType::default().with_module(source_type.is_module()));
            driver.printed.clone()
        };
        if transformed1 == transformed2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch("Mismatch", transformed1, transformed2)
        }
    }
}

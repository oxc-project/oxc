use std::borrow::Cow;

use oxc::diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_tasks_common::normalize_path;

use crate::{
    Driver,
    suite::{ExecutionOutput, ExecutionResult, LoadedTest, TestRunner},
};

/// Parser - Implements the generalized TestRunner trait
pub struct ParserRunner;

impl TestRunner for ParserRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        use crate::suite::TestMetadata;
        use crate::test262::TestFlag;

        // Handle TypeScript tests with multiple compilation units
        if let Some(units) = test.typescript_units() {
            let needs_strict = Self::needs_strict_mode(test);
            // Run all units, stop at first error
            let result = units
                .iter()
                .map(|(content, source_type)| {
                    let source_text = if needs_strict {
                        Cow::Owned(format!("'use strict';\n{content}"))
                    } else {
                        Cow::Borrowed(content.as_str())
                    };
                    Self::parse_content(&source_text, *source_type, &test.id, needs_strict)
                })
                .find(Result::is_err)
                .unwrap_or(Ok(()));

            return Some(Self::result_to_execution_result(result));
        }

        // Test262 dual-run logic: run non-strict first, then strict if needed
        // Tests without specific flags are run twice per Test262 spec
        if let TestMetadata::Test262 { flags, .. } = &test.metadata {
            // Determine run mode based on flags
            if flags.contains(&TestFlag::OnlyStrict) {
                // OnlyStrict: run with strict mode only
                let source_text = Cow::Owned(format!("'use strict';\n{}", test.code));
                let result = Self::parse_content(&source_text, test.source_type, &test.id, true);
                return Some(Self::result_to_execution_result(result));
            } else if flags.contains(&TestFlag::Module) {
                // Module: run as module (no strict prefix needed, modules are strict by default)
                let result = Self::parse_content(&test.code, test.source_type, &test.id, false);
                return Some(Self::result_to_execution_result(result));
            } else if flags.contains(&TestFlag::NoStrict) || flags.contains(&TestFlag::Raw) {
                // NoStrict or Raw: run without strict mode only
                let result = Self::parse_content(&test.code, test.source_type, &test.id, false);
                return Some(Self::result_to_execution_result(result));
            }
            // Default: run twice - first non-strict, then strict
            // This matches Test262 spec: https://github.com/tc39/test262/blob/main/INTERPRETING.md#strict-mode
            let non_strict_result =
                Self::parse_content(&test.code, test.source_type, &test.id, false);

            // Check if first run passed or correctly errored
            let should_run_strict = match &non_strict_result {
                Ok(()) => true,
                Err((_, _)) if test.should_fail => true, // CorrectError equivalent
                _ => false,
            };

            if should_run_strict {
                // Run again with strict mode
                let source_text = Cow::Owned(format!("'use strict';\n{}", test.code));
                let strict_result =
                    Self::parse_content(&source_text, test.source_type, &test.id, true);
                return Some(Self::result_to_execution_result(strict_result));
            }
            return Some(Self::result_to_execution_result(non_strict_result));
        }

        // Normal single-file tests (non-test262)
        let source_text = if Self::needs_strict_mode(test) {
            Cow::Owned(format!("'use strict';\n{}", test.code))
        } else {
            Cow::Borrowed(&test.code)
        };

        let result = Self::parse_content(
            &source_text,
            test.source_type,
            &test.id,
            Self::needs_strict_mode(test),
        );
        Some(Self::result_to_execution_result(result))
    }

    fn name(&self) -> &'static str {
        "parser"
    }
}

impl ParserRunner {
    /// Check if test needs strict mode
    fn needs_strict_mode(test: &LoadedTest) -> bool {
        use crate::suite::TestMetadata;
        use crate::test262::TestFlag;

        match &test.metadata {
            TestMetadata::Test262 { flags, .. } => flags.contains(&TestFlag::OnlyStrict),
            TestMetadata::TypeScript { settings, .. } => settings.always_strict,
            _ => false,
        }
    }

    /// Core parsing logic - reused from ParserExecutor
    fn parse_content(
        source_text: &str,
        source_type: oxc::span::SourceType,
        path_id: &str,
        _needs_strict: bool,
    ) -> Result<(), (String, bool)> {
        use std::path::PathBuf;

        let mut driver = Driver {
            path: PathBuf::from(path_id),
            allow_return_outside_function: false,
            ..Driver::default()
        };

        driver.run(source_text, source_type);
        let errors = driver.errors();

        if errors.is_empty() {
            Ok(())
        } else {
            let error_msg = Self::format_errors(&errors, path_id, source_text);
            Err((error_msg, driver.panicked))
        }
    }

    /// Format errors for output - reused from ParserExecutor
    fn format_errors(
        errors: &[oxc::diagnostics::OxcDiagnostic],
        path_id: &str,
        source_text: &str,
    ) -> String {
        let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
        let mut output = String::new();

        for error in errors {
            let error = error.clone().with_source_code(NamedSource::new(
                normalize_path(path_id),
                source_text.to_string(),
            ));
            handler.render_report(&mut output, error.as_ref()).unwrap();
        }

        output
    }

    /// Convert parse result to ExecutionResult
    fn result_to_execution_result(result: Result<(), (String, bool)>) -> ExecutionResult {
        match result {
            Ok(()) => ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            },
            Err((err, panicked)) => ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![err]),
                panicked,
            },
        }
    }
}

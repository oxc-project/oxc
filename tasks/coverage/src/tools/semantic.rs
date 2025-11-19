use std::path::Path;

use oxc::{
    span::SourceType,
    transformer::{JsxRuntime, TransformOptions},
};

use crate::{
    driver::Driver,
    suite::{ExecutionOutput, ExecutionResult, LoadedTest, ParsedTest, TestMetadata, TestRunner},
    tools::get_default_transformer_options,
};

// ============================================================================
// Special Filter - SemanticTypeScriptFilter
// ============================================================================

/// Semantic-specific TypeScript filter (skips tests with error codes)
/// This filter has special handling that cannot use the generic SkipFailingFilter.
pub struct SemanticTypeScriptFilter {
    base: crate::typescript::TypeScriptFilter,
}

impl SemanticTypeScriptFilter {
    pub const fn new() -> Self {
        Self { base: crate::typescript::TypeScriptFilter::new() }
    }
}

impl crate::suite::TestFilter for SemanticTypeScriptFilter {
    fn skip_path(&self, path: &Path) -> bool {
        self.base.skip_path(path)
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        // Semantic skips TypeScript tests with error codes
        if let TestMetadata::TypeScript { error_codes, .. } = &test.metadata
            && !error_codes.is_empty()
        {
            return true;
        }
        test.should_fail || self.base.skip_test(test)
    }
}

/// Semantic Runner - Implements the TestRunner trait
pub struct SemanticRunner;

impl TestRunner for SemanticRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        // Handle TypeScript tests with multiple compilation units
        if let Some(units) = test.typescript_units() {
            // Check if this TypeScript test has @jsx: react setting
            let has_react_jsx = if let TestMetadata::TypeScript { settings, .. } = &test.metadata {
                settings.jsx.last().is_some_and(|jsx| jsx == "react")
            } else {
                false
            };

            for (content, mut source_type) in units {
                // Only use Classic runtime when @jsx: react is set
                // Also set source_type to module for react JSX tests
                let options = if has_react_jsx {
                    source_type = source_type.with_module(true);
                    let mut opts = get_default_transformer_options(None);
                    opts.jsx.runtime = JsxRuntime::Classic;
                    Some(opts)
                } else {
                    None
                };
                let result = Self::run_semantic(&content, source_type, &test.id, options);

                // Return first error
                if result.error_kind.has_errors() || result.panicked {
                    return Some(result);
                }
            }
            // All units passed
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            });
        }

        // Normal single-file tests
        Some(Self::run_semantic(&test.code, test.source_type, &test.id, None))
    }

    fn name(&self) -> &'static str {
        "semantic"
    }
}

impl SemanticRunner {
    /// Run semantic analysis on code
    fn run_semantic(
        source_text: &str,
        source_type: SourceType,
        path_id: &str,
        options: Option<TransformOptions>,
    ) -> ExecutionResult {
        use std::path::PathBuf;

        let mut driver = Driver {
            path: PathBuf::from(path_id),
            transform: Some(options.unwrap_or_else(|| get_default_transformer_options(None))),
            check_semantic: true,
            ..Driver::default()
        };

        driver.run(source_text, source_type);
        let errors = driver.errors();

        if errors.is_empty() {
            ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            }
        } else {
            // Format errors with path prefix for snapshot format
            let error_text = format!(
                "semantic Error: tasks/coverage/{}\n{}\n",
                path_id,
                errors.into_iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("\n")
            );
            ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![error_text]),
                panicked: false, // Semantic doesn't panic, just collects errors
            }
        }
    }
}

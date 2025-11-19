use oxc::{
    allocator::Allocator,
    parser::{Parser, ParserReturn},
    span::SourceType,
};
use oxc_formatter::{FormatOptions, Formatter, get_parse_options};

use crate::suite::{ExecutionOutput, ExecutionResult, LoadedTest, TestResult, TestRunner};

/// Formatter Runner - Implements the TestRunner trait
pub struct FormatterRunner;

impl TestRunner for FormatterRunner {
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
        "formatter"
    }
}

impl FormatterRunner {
    fn run_idempotency(source_text: &str, source_type: SourceType) -> TestResult {
        let options = FormatOptions::default();

        let allocator = Allocator::default();
        let ParserReturn { program, errors, .. } =
            Parser::new(&allocator, source_text, source_type)
                .with_options(get_parse_options())
                .parse();

        if !errors.is_empty() {
            return TestResult::ParseError(
                errors.iter().map(std::string::ToString::to_string).collect(),
                false,
            );
        }

        let source_text1 = Formatter::new(&allocator, options.clone()).build(&program);

        let allocator = Allocator::default();
        let ParserReturn { program, errors, .. } =
            Parser::new(&allocator, &source_text1, source_type)
                .with_options(get_parse_options())
                .parse();

        if !errors.is_empty() {
            return TestResult::ParseError(
                errors.iter().map(std::string::ToString::to_string).collect(),
                false,
            );
        }

        let source_text2 = Formatter::new(&allocator, options).build(&program);

        if source_text1 == source_text2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch("Mismatch", source_text1, source_text2)
        }
    }
}

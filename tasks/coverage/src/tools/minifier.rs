use oxc::minifier::CompressOptions;
use oxc::span::SourceType;

use crate::{
    Driver,
    suite::{ExecutionResult, LoadedTest, ParsedTest, TestMetadata, TestResult, TestRunner},
    test262::TestFlag,
};

/// Minifier-specific Test262 filter (skips NoStrict tests)
pub struct MinifierTest262Filter {
    base: crate::test262::Test262Filter,
}
impl MinifierTest262Filter {
    pub const fn new() -> Self {
        Self { base: crate::test262::Test262Filter::new() }
    }
}
impl crate::suite::TestFilter for MinifierTest262Filter {
    fn skip_path(&self, path: &std::path::Path) -> bool {
        self.base.skip_path(path)
    }
    fn skip_test(&self, test: &ParsedTest) -> bool {
        // Skip tests that should fail
        if test.should_fail || self.base.skip_test(test) {
            return true;
        }
        // Skip noStrict tests - minifier cannot handle non-strict syntax like `with`
        if let TestMetadata::Test262 { flags, .. } = &test.metadata
            && flags.contains(&TestFlag::NoStrict)
        {
            return true;
        }
        false
    }
}

/// Minifier-specific Babel filter (skips TypeScript files)
pub struct MinifierBabelFilter {
    base: crate::babel::BabelFilter,
}
impl MinifierBabelFilter {
    pub const fn new() -> Self {
        Self { base: crate::babel::BabelFilter::new() }
    }
}
impl crate::suite::TestFilter for MinifierBabelFilter {
    fn skip_path(&self, path: &std::path::Path) -> bool {
        // Skip TypeScript files - minifier doesn't transform them
        if path.extension().is_some_and(|ext| ext == "ts" || ext == "tsx") {
            return true;
        }
        self.base.skip_path(path)
    }
    fn skip_test(&self, test: &ParsedTest) -> bool {
        // Skip tests that should fail
        if test.should_fail {
            return true;
        }
        // Skip TypeScript source types (some Babel tests have TS source type in JSON)
        if test.source_type.is_typescript() {
            return true;
        }
        self.base.skip_test(test)
    }
}

/// Minifier Runner - Implements the TestRunner trait
pub struct MinifierRunner;

impl TestRunner for MinifierRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        let result = Self::run_idempotency(&test.code, test.source_type);
        Some(result.into())
    }

    fn name(&self) -> &'static str {
        "minifier"
    }
}

impl MinifierRunner {
    fn run_idempotency(source_text: &str, source_type: SourceType) -> TestResult {
        Driver { compress: Some(CompressOptions::smallest()), codegen: true, ..Driver::default() }
            .idempotency("Compress", source_text, source_type)
    }
}

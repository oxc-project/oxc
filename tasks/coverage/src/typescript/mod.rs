pub mod constants;
mod diagnostics_code_collector;
mod meta;
mod transpile_runner;

pub use diagnostics_code_collector::save_reviewed_tsc_diagnostics_codes;

use std::path::Path;

pub use self::meta::CompilerSettings;
use self::meta::TestCaseContent;
use crate::suite::{MetadataParser, TestFilter, TestMetadata, TypeScriptUnit};

const TESTS_ROOT: &str = "typescript/tests";

pub struct TypeScriptMetadataParser;

impl MetadataParser for TypeScriptMetadataParser {
    fn parse(&self, path: &Path, code: &str) -> TestMetadata {
        // Extract from TypeScriptCase::new()
        let TestCaseContent { tests, settings, error_codes } =
            TestCaseContent::make_units_from_test(path, code);

        // Convert TestUnitData to TypeScriptUnit
        let units = tests
            .into_iter()
            .map(|unit| TypeScriptUnit {
                name: unit.name,
                content: unit.content,
                source_type: unit.source_type,
            })
            .collect();

        TestMetadata::TypeScript { units, settings, error_codes }
    }
}

/// TypeScript test filter
/// Filters based on supported paths and NOT_SUPPORTED_TEST_PATHS
pub struct TypeScriptFilter;

impl TypeScriptFilter {
    pub const fn new() -> Self {
        Self
    }
}

impl TestFilter for TypeScriptFilter {
    fn skip_path(&self, path: &Path) -> bool {
        // Extract from TypeScriptSuite::skip_test_path()
        // In coverage mode, only run conformance tests (compiler tests cause stack overflows)
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));

        let unsupported_tests =
            constants::NOT_SUPPORTED_TEST_PATHS.iter().any(|p| path.to_string_lossy().contains(p));

        !supported_paths || unsupported_tests
    }

    fn skip_test(&self, _test: &crate::suite::ParsedTest) -> bool {
        // Parser tool runs on all tests (no additional filtering)
        false
    }
}

pub use self::transpile_runner::{TranspileFilter, TranspileRunner, TranspileValidator};

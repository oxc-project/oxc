use std::path::{Path, PathBuf};

use crate::suite::{MetadataParser, PathBasedFilter, TestFilter, TestMetadata};

// ============================================================================
// ETL Architecture - Synthetic Test Generation
// ============================================================================

/// Represents a synthetic test with in-memory code
pub struct SyntheticTest {
    pub path: PathBuf,
    pub code: String,
}

/// Generate synthetic stress test cases
pub fn generate_extra_test_cases() -> Vec<SyntheticTest> {
    vec![
        SyntheticTest {
            path: PathBuf::from("huge_binary_expression.js"),
            code: String::from("a") + &"+ a".repeat(1000),
        },
        SyntheticTest {
            path: PathBuf::from("huge_nested_statements.js"),
            code: "if (true) {".repeat(1000) + &"}".repeat(1000),
        },
    ]
}

// ============================================================================
// ETL Architecture - Metadata Parser and Filter
// ============================================================================

/// Misc metadata parser
/// Simple parser - no metadata to parse, just returns Misc variant
pub struct MiscMetadataParser;

impl MetadataParser for MiscMetadataParser {
    fn parse(&self, _path: &Path, _code: &str) -> TestMetadata {
        // Misc tests have no metadata to parse
        // Source type is determined by determine_source_type() based on file extension
        TestMetadata::Misc
    }
}

/// Misc test filter
/// No special filtering - all files in misc/ are valid tests
pub struct MiscFilter {
    path_filter: PathBasedFilter,
}

impl MiscFilter {
    pub const fn new() -> Self {
        // No filtering needed for misc suite
        const EXCLUDED_DIRS: &[&str] = &[];
        const EXCLUDED_PATHS: &[&str] = &[];
        const EXCLUDED_EXTENSIONS: &[&str] = &[];

        Self {
            path_filter: PathBasedFilter::new(EXCLUDED_DIRS, EXCLUDED_PATHS, EXCLUDED_EXTENSIONS),
        }
    }
}

impl TestFilter for MiscFilter {
    fn skip_path(&self, path: &Path) -> bool {
        // No paths are skipped in misc suite
        self.path_filter.should_skip(path)
    }

    fn skip_test(&self, _test: &crate::suite::ParsedTest) -> bool {
        // No tests are skipped in misc suite
        false
    }
}

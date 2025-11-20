use std::path::Path;

use oxc::{span::SourceType, transformer::BabelOptions};
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    suite::{MetadataParser, PathBasedFilter, TestFilter, TestMetadata},
    workspace_root,
};

/// output.json
#[derive(Debug, Default, Clone, Deserialize)]
pub struct BabelOutput {
    pub errors: Option<Vec<String>>,
}

// ============================================================================
// ETL Architecture - Metadata Parser and Filter
// ============================================================================

/// Babel metadata parser
/// Extracts source type from Babel options.json
pub struct BabelMetadataParser;

impl MetadataParser for BabelMetadataParser {
    fn parse(&self, path: &Path, _code: &str) -> TestMetadata {
        // Extract from BabelCase::new()
        let dir = workspace_root().join(path);
        let options = BabelOptions::from_test_path(dir.parent().unwrap());

        let mut source_type = SourceType::from_path(path)
            .unwrap()
            .with_script(true)
            .with_jsx(options.is_jsx())
            .with_typescript(options.is_typescript())
            .with_typescript_definition(options.is_typescript_definition());

        if options.is_unambiguous() {
            source_type = source_type.with_unambiguous(true);
        } else if options.is_module() {
            source_type = source_type.with_module(true);
        }

        // Determine should_fail from output.json or options.json
        let should_fail = Self::determine_should_fail(path, &options);

        TestMetadata::Babel { source_type, should_fail }
    }
}

impl BabelMetadataParser {
    // it is an error if:
    //   * its output.json contains an errors field
    //   * the directory contains a options.json with a "throws" field
    fn determine_should_fail(path: &Path, options: &BabelOptions) -> bool {
        let output_json = BabelFilter::read_output_json(path);

        if let Some(output_json) = output_json {
            return output_json.errors.is_some_and(|errors| !errors.is_empty());
        }

        if options.throws.is_some() {
            return true;
        }

        // both files doesn't exist
        true
    }
}

/// Babel test filter
/// Filters experimental features, non-interesting tests, and incorrect extensions
pub struct BabelFilter {
    path_filter: PathBasedFilter,
}

impl BabelFilter {
    pub const fn new() -> Self {
        const EXCLUDED_DIRS: &[&str] = &[
            "experimental",
            "record-and-tuple",
            "es-record",
            "es-tuple",
            "with-pipeline",
            "v8intrinsic",
            "async-do-expression",
            "export-ns-from",
            "annex-b/disabled",
        ];

        const EXCLUDED_PATHS: &[&str] = &[
            // tests for babel options (`startIndex`, `startLine`)
            "core/categorized/invalid-startindex-and-startline-specified-without-startcolumn/input.js",
            "core/categorized/startline-and-startcolumn-specified/input.js",
            "core/categorized/startline-specified/input.js",
            // tests for babel options (`sourceType: 'commonjs'` + other options)
            "core/sourcetype-commonjs/invalid-allowAwaitOutsideFunction-false/input.js",
            "core/sourcetype-commonjs/invalid-allowNewTargetOutsideFunction-false/input.js",
            "core/sourcetype-commonjs/invalid-allowNewTargetOutsideFunction-true/input.js",
            "core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-false/input.js",
            "core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-true/input.js",
        ];

        const EXCLUDED_EXTENSIONS: &[&str] = &["json", "md"];

        Self {
            path_filter: PathBasedFilter::new(EXCLUDED_DIRS, EXCLUDED_PATHS, EXCLUDED_EXTENSIONS),
        }
    }

    /// Helper to read JSON files from test directory
    fn read_file<T>(path: &Path, file_name: &'static str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let file = path.with_file_name(file_name);
        if file.exists() {
            let file = std::fs::File::open(file).ok()?;
            let reader = std::io::BufReader::new(file);
            let json: serde_json::Result<T> = serde_json::from_reader(reader);
            return json.ok();
        }
        None
    }

    /// Read output.json or output.extended.json
    fn read_output_json(path: &Path) -> Option<BabelOutput> {
        let dir = workspace_root().join(path);
        if let Some(json) = Self::read_file::<BabelOutput>(&dir, "output.json") {
            return Some(json);
        }
        Self::read_file::<BabelOutput>(&dir, "output.extended.json")
    }
}

impl TestFilter for BabelFilter {
    fn skip_path(&self, path: &Path) -> bool {
        // Check basic path filtering first
        if self.path_filter.should_skip(path) {
            return true;
        }

        // Check for files without extensions
        path.extension().is_none()
    }

    fn skip_test(&self, test: &crate::suite::ParsedTest) -> bool {
        const NOT_SUPPORTED_PLUGINS: &[&str] =
            &["async-do-expression", "flow", "placeholders", "decorators-legacy", "recordAndTuple"];

        // Extract options to check for unsupported plugins
        let dir = workspace_root().join(&test.path);
        let options = BabelOptions::from_test_path(dir.parent().unwrap());

        let has_unsupported_plugins = options
            .plugins
            .unsupported
            .iter()
            .any(|p| NOT_SUPPORTED_PLUGINS.iter().any(|plugin| *plugin == p));

        has_unsupported_plugins
            || options.allow_await_outside_function
            || options.allow_undeclared_exports
    }
}

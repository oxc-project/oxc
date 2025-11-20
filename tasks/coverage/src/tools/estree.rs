use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use similar::TextDiff;

use oxc::{
    allocator::Allocator,
    ast_visit::utf8_to_utf16::Utf8ToUtf16,
    diagnostics::OxcDiagnostic,
    parser::{ParseOptions, Parser},
    span::SourceType,
};

use crate::{suite::TestResult, workspace_root};

use crate::suite::{
    ExecutionOutput, ExecutionResult, LoadedTest, ParsedTest, TestFilter, TestRunner,
};

// ============================================================================
// ETL Architecture - Filters
// ============================================================================

/// EstreeTest262Filter - Skips tests that ESTree can't handle
pub struct EstreeTest262Filter {
    base: crate::test262::Test262Filter,
}

impl EstreeTest262Filter {
    pub const fn new() -> Self {
        Self { base: crate::test262::Test262Filter::new() }
    }
}

impl TestFilter for EstreeTest262Filter {
    fn skip_path(&self, path: &Path) -> bool {
        // Skip hashbang tests - ESTree doesn't handle them
        if path.to_string_lossy().contains("language/comments/hashbang") {
            return true;
        }
        self.base.skip_path(path)
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        self.base.skip_test(test)
    }
}

/// EstreeTypescriptFilter - Skips tests with known issues
pub struct EstreeTypescriptFilter {
    base: crate::typescript::TypeScriptFilter,
}

impl EstreeTypescriptFilter {
    pub const fn new() -> Self {
        Self { base: crate::typescript::TypeScriptFilter::new() }
    }
}

impl TestFilter for EstreeTypescriptFilter {
    fn skip_path(&self, path: &Path) -> bool {
        self.base.skip_path(path)
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        // Skip cases which are failing in parser conformance tests.
        // Some of these should parse correctly, but the cause is not related to ESTree serialization.
        const PARSE_ERROR_PATHS: &[&str] = &[
            // Fails because fixture is not loaded as an ESM module (bug in tester)
            "typescript/tests/cases/compiler/arrayFromAsync.ts",
            // Differences between TS's recoverable parser and Oxc's non-recoverable parser
            "typescript/tests/cases/conformance/classes/propertyMemberDeclarations/staticPropertyNameConflicts.ts",
            "typescript/tests/cases/conformance/es2019/importMeta/importMeta.ts",
            // Decorators - probably should be parsed correctly (bug in parser)
            "typescript/tests/cases/compiler/sourceMapValidationDecorators.ts",
            "typescript/tests/cases/conformance/esDecorators/esDecorators-decoratorExpression.1.ts",
        ];

        // Skip tests where `@typescript-eslint/parser` is incorrect
        const INCORRECT_PATHS: &[&str] = &[
            // TS-ESLint includes `\r` in `raw` field of `TemplateElement`.
            "typescript/tests/cases/conformance/es6/templates/templateStringMultiline3.ts",
        ];

        // Skip tests where fixture starts with a hashbang
        const HASHBANG_PATHS: &[&str] = &[
            "typescript/tests/cases/compiler/emitBundleWithShebang1.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebang2.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebangAndPrologueDirectives1.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebangAndPrologueDirectives2.ts",
            "typescript/tests/cases/compiler/shebang.ts",
            "typescript/tests/cases/compiler/shebangBeforeReferences.ts",
        ];

        let path_str = test.path.to_string_lossy();

        // Skip specific problematic paths (exact match)
        let all_skip_paths: &[&str] = &[
            PARSE_ERROR_PATHS[0],
            PARSE_ERROR_PATHS[1],
            PARSE_ERROR_PATHS[2],
            PARSE_ERROR_PATHS[3],
            PARSE_ERROR_PATHS[4],
            INCORRECT_PATHS[0],
            HASHBANG_PATHS[0],
            HASHBANG_PATHS[1],
            HASHBANG_PATHS[2],
            HASHBANG_PATHS[3],
            HASHBANG_PATHS[4],
            HASHBANG_PATHS[5],
        ];

        if all_skip_paths.iter().any(|p| path_str.contains(p)) {
            return true;
        }

        // Skip tests that should fail
        if test.should_fail {
            return true;
        }

        self.base.skip_test(test)
    }
}

/// ESTree Test262 runner
/// Parses source, converts to ESTree JSON, returns string output for comparison
pub struct EstreeTest262Runner;

impl TestRunner for EstreeTest262Runner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        let source_text = &test.code;
        let source_type = test.source_type;

        // Parse with oxc
        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type).parse();

        // Handle parse errors
        if ret.panicked || !ret.errors.is_empty() {
            let error =
                ret.errors.first().map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![error]),
                panicked: ret.panicked,
            });
        }

        // Convert to UTF-16 spans (ESTree spec requirement)
        let mut program = ret.program;
        Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);

        // Serialize to ESTree JSON
        let oxc_json = program.to_pretty_estree_js_json(false);

        // Return JSON output for comparison
        Some(ExecutionResult {
            output: ExecutionOutput::String(oxc_json),
            error_kind: crate::suite::ErrorKind::None,
            panicked: false,
        })
    }

    fn name(&self) -> &'static str {
        "estree_test262"
    }
}

/// ESTree JSX runner
pub struct EstreeJsxRunner;

impl TestRunner for EstreeJsxRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        let source_text = &test.code;
        let source_type = SourceType::default().with_jsx(true);

        // Parse with oxc
        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type).parse();

        // Handle parse errors
        if ret.panicked || !ret.errors.is_empty() {
            let error =
                ret.errors.first().map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![error]),
                panicked: ret.panicked,
            });
        }

        // Convert to UTF-16 spans
        let mut program = ret.program;
        Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);

        // Serialize to ESTree JSON
        let oxc_json = program.to_pretty_estree_js_json(false);

        // Return JSON output for comparison
        Some(ExecutionResult {
            output: ExecutionOutput::String(oxc_json),
            error_kind: crate::suite::ErrorKind::None,
            panicked: false,
        })
    }

    fn name(&self) -> &'static str {
        "estree_jsx"
    }
}

/// ESTree TypeScript runner
/// Handles TypeScript files with multiple compilation units
pub struct EstreeTypescriptRunner;

impl TestRunner for EstreeTypescriptRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        use crate::suite::TestMetadata;

        // Handle TypeScript tests with multiple compilation units
        if let TestMetadata::TypeScript { units, .. } = &test.metadata {
            let mut json_outputs = Vec::new();

            // Process all units
            for unit in units {
                let source_text = &unit.content;
                let source_type = unit.source_type;

                // Parse with oxc (preserve_parens: false to match TS-ESLint)
                let allocator = Allocator::new();
                let options = ParseOptions { preserve_parens: false, ..Default::default() };
                let ret =
                    Parser::new(&allocator, source_text, source_type).with_options(options).parse();

                // Handle parse errors
                if ret.panicked || !ret.errors.is_empty() {
                    let error = ret
                        .errors
                        .first()
                        .map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
                    // Add trailing newline for TypeScript tests
                    return Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Errors(vec![error + "\n"]),
                        panicked: ret.panicked,
                    });
                }

                // Convert to UTF-16 spans
                let mut program = ret.program;
                Utf8ToUtf16::new(source_text)
                    .convert_program_with_ascending_order_checks(&mut program);

                // Serialize to ESTree TypeScript JSON
                let oxc_json = program.to_pretty_estree_ts_json(false);
                json_outputs.push(oxc_json);
            }

            // Concatenate all outputs for comparison
            // Each unit's output is separated by a newline
            let combined_output = json_outputs.join("\n");
            return Some(ExecutionResult {
                output: ExecutionOutput::String(combined_output),
                error_kind: crate::suite::ErrorKind::None,
                panicked: false,
            });
        }

        // Shouldn't reach here, but handle gracefully
        Some(ExecutionResult {
            output: ExecutionOutput::None,
            error_kind: crate::suite::ErrorKind::Errors(vec!["Not a TypeScript test".to_string()]),
            panicked: false,
        })
    }

    fn name(&self) -> &'static str {
        "estree_typescript"
    }
}

// ============================================================================
// Expected Output Loaders - Load test data + expected JSON outputs
// ============================================================================

use crate::{
    discovery::FileDiscovery,
    suite::{ExpectedOutput, MetadataParser, TestDescriptor, TestLoader, TestMetadata},
};

/// ESTree Test262 Loader - Loads test + expected JSON from acorn-test262 submodule
pub struct EstreeTest262Loader<P: MetadataParser> {
    metadata_parser: P,
}

impl<P: MetadataParser> EstreeTest262Loader<P> {
    pub fn new(metadata_parser: P) -> Self {
        Self { metadata_parser }
    }

    /// Map source path to expected JSON path
    /// e.g., test262/test/foo.js → acorn-test262/tests/test262/test/foo.json
    fn get_expected_path(source_path: &Path) -> PathBuf {
        workspace_root().join("acorn-test262/tests").join(source_path).with_extension("json")
    }
}

impl<P: MetadataParser> TestLoader for EstreeTest262Loader<P> {
    fn load(&self, descriptor: &TestDescriptor) -> Option<LoadedTest> {
        let TestDescriptor::FilePath(path) = descriptor else {
            return None; // ESTree Test262 loader only handles file paths
        };

        // Read source file
        let code = FileDiscovery::read_file(path).ok()?;

        // Parse metadata
        let metadata = self.metadata_parser.parse(path, &code);
        let source_type = metadata.determine_source_type(path);

        // Determine should_fail from metadata
        let should_fail = metadata.should_fail();

        // Load expected JSON output - skip tests without expected output
        let expected_path = Self::get_expected_path(path);
        let expected = if fs::exists(&expected_path).ok()? {
            match fs::read_to_string(&expected_path) {
                Ok(json) => ExpectedOutput::String(json),
                Err(_) => return None, // Skip tests with read errors
            }
        } else {
            return None; // Skip tests with no expected output
        };

        Some(LoadedTest {
            id: path.to_string_lossy().into_owned(),
            code,
            metadata,
            source_type,
            should_fail,
            expected,
        })
    }
}

/// ESTree JSX Loader - Loads JSX test + expected JSON from acorn-test262 submodule
pub struct EstreeJsxLoader; // No metadata parser needed - JSX tests are simple

impl EstreeJsxLoader {
    pub const fn new() -> Self {
        Self
    }

    /// Map source path to expected JSON path
    /// e.g., acorn-test262/tests/acorn-jsx/foo.jsx → acorn-test262/tests/acorn-jsx/foo.json
    fn get_expected_path(source_path: &Path) -> PathBuf {
        workspace_root().join(source_path.with_extension("json"))
    }
}

impl TestLoader for EstreeJsxLoader {
    fn load(&self, descriptor: &TestDescriptor) -> Option<LoadedTest> {
        let TestDescriptor::FilePath(path) = descriptor else {
            return None; // JSX loader only handles file paths
        };

        // Read source file
        let code = FileDiscovery::read_file(path).ok()?;

        // Determine source type (module + jsx)
        let source_type = SourceType::default().with_module(true).with_jsx(true);

        // Determine should_fail from directory name
        let should_fail = path.parent()?.file_name()? == "fail";

        // Load expected JSON output - skip tests without expected output
        let expected_path = Self::get_expected_path(path);
        let expected = if fs::exists(&expected_path).ok()? {
            match fs::read_to_string(&expected_path) {
                Ok(json) => ExpectedOutput::String(json),
                Err(_) => return None, // Skip tests with read errors
            }
        } else {
            return None; // Skip tests with no expected output
        };

        Some(LoadedTest {
            id: path.to_string_lossy().into_owned(),
            code,
            metadata: TestMetadata::Misc,
            source_type,
            should_fail,
            expected,
        })
    }
}

/// ESTree TypeScript Loader - Loads TypeScript test + expected markdown JSON
pub struct EstreeTypescriptLoader<P: MetadataParser> {
    metadata_parser: P,
}

impl<P: MetadataParser> EstreeTypescriptLoader<P> {
    pub fn new(metadata_parser: P) -> Self {
        Self { metadata_parser }
    }

    /// Map source path to expected markdown path
    /// e.g., typescript/tests/cases/foo.ts → acorn-test262/tests/typescript/tests/cases/foo.ts.md
    fn get_expected_path(source_path: &Path) -> Option<PathBuf> {
        let extension = source_path.extension()?.to_str()?;
        Some(
            workspace_root()
                .join("acorn-test262/tests")
                .join(source_path)
                .with_extension(format!("{extension}.md")),
        )
    }

    /// Parse markdown file to extract JSON blocks
    /// Format: __ESTREE_TEST__:PASS:\n```json\n{...}\n```\n
    fn parse_markdown(content: &str) -> Vec<String> {
        content
            .split("__ESTREE_TEST__")
            .skip(1)
            .filter_map(|s| {
                let s = s.strip_prefix(":PASS:\n```json\n")?;
                s.strip_suffix("\n```\n")
            })
            .map(String::from)
            .collect()
    }
}

impl<P: MetadataParser> TestLoader for EstreeTypescriptLoader<P> {
    fn load(&self, descriptor: &TestDescriptor) -> Option<LoadedTest> {
        let TestDescriptor::FilePath(path) = descriptor else {
            return None; // TypeScript loader only handles file paths
        };

        // Read source file
        let code = FileDiscovery::read_file(path).ok()?;

        // Parse metadata (includes TypeScript units)
        let metadata = self.metadata_parser.parse(path, &code);
        let source_type = metadata.determine_source_type(path);

        // Determine should_fail from metadata
        let should_fail = metadata.should_fail();

        // Load expected markdown file and extract JSON blocks - skip tests without expected output
        let expected_path = Self::get_expected_path(path)?;
        let expected = if fs::exists(&expected_path).ok()? {
            match fs::read_to_string(&expected_path) {
                Ok(markdown) => {
                    let json_blocks = Self::parse_markdown(&markdown);

                    if json_blocks.is_empty() {
                        return None; // Skip tests with no JSON blocks
                    }

                    // Concatenate all JSON blocks for multi-unit comparison
                    ExpectedOutput::String(json_blocks.join("\n"))
                }
                Err(_) => return None, // Skip tests with read errors
            }
        } else {
            return None; // Skip tests with no expected output file
        };

        Some(LoadedTest {
            id: path.to_string_lossy().into_owned(),
            code,
            metadata,
            source_type,
            should_fail,
            expected,
        })
    }
}

// ============================================================================
// Comparison Validator - Compares actual vs expected JSON
// ============================================================================

use crate::suite::ResultValidator;

/// Comparison validator for ESTree JSON outputs
/// Compares actual output with expected JSON, optionally writing diffs
pub struct EstreeComparisonValidator {
    /// Enable diff file generation (disabled in CI)
    write_diffs: bool,
}

impl EstreeComparisonValidator {
    pub fn new() -> Self {
        // Disable diff writing in CI
        let write_diffs = std::option_env!("CI") != Some("true");
        Self { write_diffs }
    }

    /// Write diff to acorn-test262-diff directory for local debugging
    fn write_diff(&self, test_path: &str, actual: &str, expected: &str) {
        if !self.write_diffs {
            return;
        }

        let diff_path =
            Path::new("./tasks/coverage/acorn-test262-diff").join(test_path).with_extension("diff");

        if let Some(parent) = diff_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(mut file) = fs::File::create(&diff_path) {
            let text_diff = TextDiff::from_lines(expected, actual);
            let diff_str = format!("{}", text_diff.unified_diff().missing_newline_hint(false));
            let _ = write!(file, "{diff_str}");
        }
    }
}

impl Default for EstreeComparisonValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultValidator for EstreeComparisonValidator {
    fn validate(&self, test: &LoadedTest, result: ExecutionResult) -> TestResult {
        // Handle execution errors first
        let has_errors = result.error_kind.has_errors() || result.panicked;

        if has_errors {
            let error_msg = match &result.error_kind {
                crate::suite::ErrorKind::Errors(errors) => errors.join("\n"),
                crate::suite::ErrorKind::Mismatch { actual, expected, .. } => {
                    format!("Mismatch:\nActual: {actual}\nExpected: {expected}")
                }
                crate::suite::ErrorKind::Generic { error, .. } => error.clone(),
                crate::suite::ErrorKind::None => String::new(),
            };
            return if test.should_fail {
                TestResult::CorrectError(error_msg, result.panicked)
            } else {
                TestResult::ParseError(error_msg, result.panicked)
            };
        }

        // For should_fail tests that didn't error, they incorrectly passed
        if test.should_fail {
            return TestResult::IncorrectlyPassed;
        }

        // Extract actual output
        let actual_output = match &result.output {
            ExecutionOutput::String(s) => s,
            ExecutionOutput::None => {
                // No output generated (shouldn't happen for ESTree)
                return TestResult::Passed;
            }
            ExecutionOutput::MultiPhase(_) => {
                // Multi-phase not yet supported for ESTree
                return TestResult::GenericError(
                    "estree",
                    "Multi-phase output not supported".to_string(),
                );
            }
        };

        // Extract expected output
        let expected_output = match &test.expected {
            ExpectedOutput::String(s) => s,
            ExpectedOutput::None => {
                // No expected output - skip comparison (test was filtered)
                return TestResult::Passed;
            }
            ExpectedOutput::Error { .. } => {
                // Expected an error but got success
                return TestResult::IncorrectlyPassed;
            }
        };

        // Compare JSON outputs
        if actual_output == expected_output {
            TestResult::Passed
        } else {
            // Mismatch - write diff for debugging
            self.write_diff(&test.id, actual_output, expected_output);
            TestResult::Mismatch("Mismatch", actual_output.clone(), expected_output.clone())
        }
    }
}

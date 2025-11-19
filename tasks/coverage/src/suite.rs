use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use oxc::span::SourceType;

use crate::test262::{Negative, Phase, TestFlag};

#[derive(Debug, PartialEq, Eq)]
pub enum TestResult {
    Passed,
    IncorrectlyPassed,
    Mismatch(/* case */ &'static str, /* actual */ String, /* expected */ String),
    ParseError(String, /* panicked */ bool),
    CorrectError(String, /* panicked */ bool),
    GenericError(/* case */ &'static str, /* error */ String),
}

// ============================================================================
// ETL Architecture - Clean Separation of Concerns
// ============================================================================

/// Suite-specific metadata (decouples metadata from Case trait)
#[derive(Debug, Clone)]
pub enum TestMetadata {
    Test262 {
        esid: Option<Box<str>>,
        features: Box<[Box<str>]>,
        includes: Box<[Box<str>]>,
        flags: Box<[TestFlag]>,
        negative: Option<Negative>,
    },
    Babel {
        source_type: SourceType,
        should_fail: bool,
    },
    TypeScript {
        // TypeScript tests may have multiple compilation units
        units: Vec<TypeScriptUnit>,
        // Compiler settings from @xxx: comments (e.g., @declaration: true)
        settings: crate::typescript::CompilerSettings,
        // Expected error codes from test file (e.g., TS1234)
        error_codes: Vec<String>,
    },
    Misc,
}

#[derive(Debug, Clone)]
pub struct TypeScriptUnit {
    pub name: String,
    pub content: String,
    pub source_type: SourceType,
}

impl TestMetadata {
    /// Common interface: should this test expect to fail parsing?
    pub fn should_fail(&self) -> bool {
        match self {
            Self::Test262 { negative, .. } => {
                negative.as_ref().filter(|n| n.phase == Phase::Parse).is_some()
            }
            Self::Babel { should_fail, .. } => *should_fail,
            Self::TypeScript { error_codes, .. } => {
                // If there are still error codes to be supported, it should fail
                use crate::typescript::constants::NOT_SUPPORTED_ERROR_CODES;
                error_codes.iter().any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(code.as_str()))
            }
            Self::Misc => false,
        }
    }

    /// Determine source type from metadata and path
    pub fn determine_source_type(&self, path: &Path) -> SourceType {
        match self {
            Self::Test262 { flags, .. } => {
                if flags.contains(&TestFlag::Module) {
                    SourceType::mjs()
                } else {
                    SourceType::cjs()
                }
            }
            Self::Babel { source_type, .. } => *source_type,
            Self::TypeScript { .. } => {
                // TypeScript source type based on extension
                SourceType::ts()
            }
            Self::Misc => {
                // Misc tests use SourceType::from_path which handles all extensions
                // (.js, .mjs, .cjs, .ts, .tsx, .d.ts, etc.)
                SourceType::from_path(path).unwrap()
            }
        }
    }
}

/// A parsed test case (ETL Transform output)
#[derive(Debug)]
pub struct ParsedTest {
    pub path: PathBuf,
    pub code: String,
    pub source_type: SourceType,
    pub should_fail: bool,
    pub metadata: TestMetadata,
}

/// An executed test with result (ETL Execute output)
#[derive(Debug)]
pub struct ExecutedTest {
    pub path: PathBuf,
    pub should_fail: bool,
    pub result: TestResult,
}

impl ExecutedTest {
    /// For consistency with Case trait (used in coverage reporting)
    pub fn test_passed(&self) -> bool {
        matches!(self.result, TestResult::Passed | TestResult::CorrectError(_, _))
    }

    pub fn test_parsed(&self) -> bool {
        // "Parsed" means the parser successfully created an AST without panicking
        // Everything except panics counts as parsed
        match &self.result {
            TestResult::ParseError(_, panicked) | TestResult::CorrectError(_, panicked) => {
                !panicked
            }
            _ => true,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn test_result(&self) -> &TestResult {
        &self.result
    }

    pub fn should_fail(&self) -> bool {
        self.should_fail
    }

    pub fn error_message(&self) -> Option<&str> {
        match &self.result {
            TestResult::ParseError(msg, _) | TestResult::CorrectError(msg, _) => Some(msg.as_str()),
            TestResult::IncorrectlyPassed => Some("Test should have failed but passed"),
            _ => None,
        }
    }
}

// ============================================================================
// ETL Traits - Single Responsibility
// ============================================================================

/// Parses test metadata from source code
/// Single responsibility: Extract metadata
pub trait MetadataParser: Send + Sync {
    /// Parse metadata from source code (YAML frontmatter, JSON sidecar, etc.)
    fn parse(&self, path: &Path, code: &str) -> TestMetadata;
}

/// Filters tests based on suite and tool requirements
/// Single responsibility: Test filtering logic
pub trait TestFilter: Send + Sync {
    /// Check if a file path should be skipped during discovery
    fn skip_path(&self, path: &Path) -> bool;

    /// Check if a parsed test should be skipped before execution
    fn skip_test(&self, test: &ParsedTest) -> bool;
}

// ============================================================================
// Common Filter Implementations
// ============================================================================

/// Configurable path-based filter using string matching
pub struct PathBasedFilter {
    /// Directory names to exclude (e.g., "staging", "experimental")
    dirs: &'static [&'static str],
    /// File path patterns to exclude (e.g., specific test files)
    paths: &'static [&'static str],
    /// File extensions to exclude (e.g., "json", "md")
    extensions: &'static [&'static str],
}

impl PathBasedFilter {
    pub const fn new(
        dirs: &'static [&'static str],
        paths: &'static [&'static str],
        extensions: &'static [&'static str],
    ) -> Self {
        Self { dirs, paths, extensions }
    }

    /// Check if path should be skipped based on configuration
    pub fn should_skip(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let normalized = path_str.cow_replace('\\', "/");

        // Check excluded directories
        let in_excluded_dir = self.dirs.iter().any(|dir| normalized.contains(dir));

        // Check excluded file paths (use contains to match patterns like _FIXTURE anywhere in path)
        let is_excluded_path = self.paths.iter().any(|pattern| normalized.contains(pattern));

        // Check excluded extensions
        let has_excluded_ext = self
            .extensions
            .iter()
            .any(|ext| path.extension().is_some_and(|path_ext| path_ext == *ext));

        in_excluded_dir || is_excluded_path || has_excluded_ext
    }
}

/// Generic filter that skips tests marked as "should_fail"
///
/// This is a composable filter that wraps any base filter and adds
/// the common pattern of skipping tests with `test.should_fail`.
/// Most tools (codegen, formatter, transformer, semantic, minifier) use this pattern.
pub struct SkipFailingFilter<T: TestFilter> {
    base: T,
}

impl<T: TestFilter> SkipFailingFilter<T> {
    pub const fn new(base: T) -> Self {
        Self { base }
    }
}

impl<T: TestFilter> TestFilter for SkipFailingFilter<T> {
    fn skip_path(&self, path: &Path) -> bool {
        self.base.skip_path(path)
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        test.should_fail || self.base.skip_test(test)
    }
}

// ============================================================================
// ETL Architecture
// ============================================================================
//
// Composable architecture that supports:
// - Multiple test sources (filesystem, JSON, hybrid)
// - Sync and async execution
// - Multi-phase execution (Runtime tests)
// - Comparison-based validation (ESTree tests)
//
// The architecture separates concerns into 4 pluggable traits:
// 1. TestSource - Where tests come from
// 2. TestLoader - How to load test data
// 3. TestRunner - How to execute the tool
// 4. ResultValidator - How to validate results

/// Descriptor for a test case
#[derive(Debug, Clone)]
pub enum TestDescriptor {
    /// Physical file path
    FilePath(PathBuf),
    /// Synthetic identifier (e.g., "ES6/arrow_functions" from JSON)
    Synthetic { id: String, code: String },
}

/// Expected output for comparison-based tests
#[derive(Debug, Clone)]
pub enum ExpectedOutput {
    /// Expected output as string (ESTree JSON)
    String(String),
    /// Expected error pattern
    #[expect(dead_code)]
    Error { phase: Phase, error_type: String },
    /// No expected output (binary pass/fail)
    None,
}

/// Loaded test with all data needed for execution
#[derive(Debug)]
pub struct LoadedTest {
    /// Unique identifier (path or synthetic ID)
    pub id: String,
    /// Source code to test
    pub code: String,
    /// Parsed metadata
    pub metadata: TestMetadata,
    /// Source type for parsing
    pub source_type: SourceType,
    /// Should this test fail?
    pub should_fail: bool,
    /// Expected output for comparison tests
    pub expected: ExpectedOutput,
}

impl LoadedTest {
    /// Extract TypeScript compilation units (content and source type)
    pub fn typescript_units(&self) -> Option<Vec<(String, SourceType)>> {
        match &self.metadata {
            TestMetadata::TypeScript { units, .. } => {
                Some(units.iter().map(|u| (u.content.clone(), u.source_type)).collect())
            }
            _ => None,
        }
    }

    /// Extract TypeScript units with compiler settings (for transformer)
    pub fn typescript_units_with_settings(
        &self,
    ) -> Option<(Vec<(String, SourceType)>, crate::typescript::CompilerSettings)> {
        match &self.metadata {
            TestMetadata::TypeScript { units, settings, .. } => {
                let unit_data = units.iter().map(|u| (u.content.clone(), u.source_type)).collect();
                Some((unit_data, settings.clone()))
            }
            _ => None,
        }
    }
}

/// Result of a single execution phase
#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct PhaseResult {
    /// Name of this phase
    pub phase_name: &'static str,
    /// Output produced by this phase
    pub output: String,
    /// Whether this phase succeeded
    pub success: bool,
}

/// Output from test execution
#[derive(Debug, Clone)]
pub enum ExecutionOutput {
    /// No output (parser tests - just success/failure)
    None,
    /// String output (codegen, ESTree JSON)
    String(String),
    /// Multiple outputs from multi-phase execution
    #[expect(dead_code)]
    MultiPhase(Vec<PhaseResult>),
}

/// Typed error representation for execution results
/// Replaces marker strings with proper type safety
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// No errors
    None,
    /// Simple error messages (parse errors, etc.)
    Errors(Vec<String>),
    /// Mismatch between actual and expected output
    Mismatch { case: &'static str, actual: String, expected: String },
    /// Generic error with context
    Generic { case: &'static str, error: String },
}

impl ErrorKind {
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !matches!(self, ErrorKind::None)
    }
}

/// Result of executing a test
#[derive(Debug)]
pub struct ExecutionResult {
    /// Output produced by execution
    pub output: ExecutionOutput,
    /// Typed error information
    pub error_kind: ErrorKind,
    /// Whether execution panicked
    pub panicked: bool,
}

impl From<TestResult> for ExecutionResult {
    fn from(result: TestResult) -> Self {
        match result {
            TestResult::Passed => {
                Self { output: ExecutionOutput::None, error_kind: ErrorKind::None, panicked: false }
            }
            TestResult::ParseError(msg, panicked) | TestResult::CorrectError(msg, panicked) => {
                Self {
                    output: ExecutionOutput::None,
                    error_kind: ErrorKind::Errors(vec![msg]),
                    panicked,
                }
            }
            TestResult::Mismatch(case, actual, expected) => Self {
                output: ExecutionOutput::None,
                error_kind: ErrorKind::Mismatch { case, actual, expected },
                panicked: false,
            },
            TestResult::GenericError(case, error) => Self {
                output: ExecutionOutput::None,
                error_kind: ErrorKind::Generic { case, error },
                panicked: false,
            },
            TestResult::IncorrectlyPassed => Self {
                output: ExecutionOutput::None,
                error_kind: ErrorKind::Errors(vec!["Unexpected test result".to_string()]),
                panicked: false,
            },
        }
    }
}

// ============================================================================
// Generalized Traits
// ============================================================================

/// Discovers test cases from a source
pub trait TestSource: Send + Sync {
    /// Discover test cases (paths, synthetic IDs, etc.)
    fn discover(&self, filter: Option<&str>) -> Vec<TestDescriptor>;
}

/// Loads test data from a descriptor
pub trait TestLoader: Send + Sync {
    /// Load test data (code, metadata, expected outputs)
    fn load(&self, descriptor: &TestDescriptor) -> Option<LoadedTest>;
}

/// Executes a tool on a test (sync or async)
pub trait TestRunner: Send + Sync {
    /// Execute synchronously (default implementation returns None)
    fn execute_sync(&self, _test: &LoadedTest) -> Option<ExecutionResult> {
        None
    }

    /// Execute asynchronously (default implementation delegates to sync)
    fn execute_async<'a>(
        &'a self,
        test: &'a LoadedTest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ExecutionResult>> + Send + 'a>>
    {
        Box::pin(async move { self.execute_sync(test) })
    }

    /// Name of this runner for reporting
    #[expect(dead_code)]
    fn name(&self) -> &'static str;
}

/// Validates execution results and produces test outcome
pub trait ResultValidator: Send + Sync {
    /// Validate result and return test outcome
    fn validate(&self, test: &LoadedTest, result: ExecutionResult) -> TestResult;
}

// ============================================================================
// Default Implementations
// ============================================================================

/// Binary pass/fail validator (most common case)
///
/// This validator handles the common case of tests that either pass or fail.
/// Uses typed ErrorKind for proper error handling without string parsing.
pub struct BinaryValidator;

impl ResultValidator for BinaryValidator {
    fn validate(&self, test: &LoadedTest, result: ExecutionResult) -> TestResult {
        match result.error_kind {
            ErrorKind::None => {
                if test.should_fail {
                    TestResult::IncorrectlyPassed
                } else {
                    TestResult::Passed
                }
            }
            ErrorKind::Errors(errors) => {
                let error_msg = errors.join("\n");
                if test.should_fail {
                    TestResult::CorrectError(error_msg, result.panicked)
                } else {
                    TestResult::ParseError(error_msg, result.panicked)
                }
            }
            ErrorKind::Mismatch { case, actual, expected } => {
                TestResult::Mismatch(case, actual, expected)
            }
            ErrorKind::Generic { case, error } => TestResult::GenericError(case, error),
        }
    }
}

#![expect(clippy::print_stdout, clippy::disallowed_methods)]

// Core
pub mod discovery;
mod pipeline;
mod registry;
mod reporter;
mod runtime;
mod suite;
// Suites
mod babel;
mod misc;
mod node_compat_table;
mod test262;
mod typescript;

mod driver;
mod tools;

use std::path::{Path, PathBuf};

use oxc_tasks_common::project_root;

use crate::{
    babel::{BabelFilter, BabelMetadataParser},
    discovery::FileDiscovery,
    driver::Driver,
    misc::{MiscFilter, MiscMetadataParser, generate_extra_test_cases},
    node_compat_table::{NodeCompatFilter, NodeCompatLoader, NodeCompatRunner, NodeCompatSource},
    registry::{StandardFilters, SuiteConfig, ToolSuites},
    reporter::Reporter,
    suite::{BinaryValidator, ExecutedTest, SkipFailingFilter, TestFilter, TestResult, TestRunner},
    test262::{Test262Filter, Test262MetadataParser},
    tools::{
        codegen::CodegenRunner,
        estree::{
            EstreeComparisonValidator, EstreeJsxLoader, EstreeJsxRunner, EstreeTest262Filter,
            EstreeTest262Loader, EstreeTest262Runner, EstreeTypescriptFilter,
            EstreeTypescriptLoader, EstreeTypescriptRunner,
        },
        formatter::FormatterRunner,
        minifier::{MinifierBabelFilter, MinifierRunner, MinifierTest262Filter},
        parser::ParserRunner,
        semantic::{SemanticRunner, SemanticTypeScriptFilter},
        transformer::TransformerRunner,
    },
    typescript::{TypeScriptFilter, TypeScriptMetadataParser, save_reviewed_tsc_diagnostics_codes},
};

pub fn workspace_root() -> PathBuf {
    project_root().join("tasks").join("coverage")
}

fn snap_root() -> PathBuf {
    workspace_root().join("snapshots")
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub debug: bool,
    pub filter: Option<String>,
    pub detail: bool,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    /// Create a reporter with current configuration
    fn reporter(&self) -> Reporter {
        Reporter::new(self.should_print_detail(), self.filter.is_none())
    }

    pub fn run_default(&self) {
        // Process all tools using pure ETL architecture with snapshot writing
        // Architecture: Pure ETL (Extract → Transform → Load) with snapshot integration
        // - Each tool runs all 4 suites (test262, babel, typescript, misc) sequentially
        // - Results are written to snapshot files for conformance tracking
        self.run_parser();
        self.run_semantic();
        self.run_codegen();
        self.run_formatter();
        self.run_transformer();
        self.run_minifier();
        self.run_transpiler();
        self.run_estree();
        self.run_nodecompat();
    }

    /// Run a tool across multiple test suites
    fn run_suites(&self, runner: &dyn TestRunner, suites: Vec<SuiteConfig>) {
        let validator = BinaryValidator;
        let reporter = self.reporter();

        for suite in suites {
            let paths = FileDiscovery::new(suite.root_path.clone()).discover_paths(
                suite.name.split('_').next_back().unwrap_or(&suite.name),
                self.filter.as_deref(),
                &|path| suite.filter.skip_path(path),
            );

            let mut results =
                pipeline::run_from_paths(&paths, suite.parser, suite.filter, runner, &validator);

            // Add synthetic tests for misc suite when no filter is applied
            if suite.include_synthetic && self.filter.is_none() {
                let synthetic_tests = generate_extra_test_cases();
                let synthetic_results = pipeline::run_from_synthetic(
                    &synthetic_tests,
                    suite.parser,
                    suite.filter,
                    runner,
                    &validator,
                );
                results.extend(synthetic_results);
            }

            reporter.report(&suite.name, &results, suite.display_path);
        }
    }

    /// Parser conformance tests
    pub fn run_parser(&self) {
        let test262_filter = Test262Filter::new();
        let babel_filter = BabelFilter::new();
        let typescript_filter = TypeScriptFilter::new();
        let misc_filter = MiscFilter::new();

        let suites = ToolSuites::new("parser")
            .add(&registry::TEST262, &Test262MetadataParser, &test262_filter)
            .add(&registry::BABEL, &BabelMetadataParser, &babel_filter)
            .add(&registry::TYPESCRIPT, &TypeScriptMetadataParser, &typescript_filter)
            .add(&registry::MISC, &MiscMetadataParser, &misc_filter)
            .build();

        self.run_suites(&ParserRunner, suites);

        // Collect reviewed TSC diagnostics codes
        save_reviewed_tsc_diagnostics_codes();
    }

    /// Semantic conformance tests
    pub fn run_semantic(&self) {
        let test262_filter = SkipFailingFilter::new(Test262Filter::new());
        let babel_filter = SkipFailingFilter::new(BabelFilter::new());
        // SemanticTypeScriptFilter has special error code handling
        let typescript_filter = SemanticTypeScriptFilter::new();
        let misc_filter = SkipFailingFilter::new(MiscFilter::new());

        let suites = ToolSuites::new("semantic")
            .add(&registry::TEST262, &Test262MetadataParser, &test262_filter)
            .add(&registry::BABEL, &BabelMetadataParser, &babel_filter)
            .add(&registry::TYPESCRIPT, &TypeScriptMetadataParser, &typescript_filter)
            .add(&registry::MISC, &MiscMetadataParser, &misc_filter)
            .build();

        self.run_suites(&SemanticRunner, suites);
    }

    /// Codegen conformance tests
    pub fn run_codegen(&self) {
        let filters = StandardFilters::new();

        let suites = ToolSuites::new("codegen")
            .add(&registry::TEST262, &Test262MetadataParser, &filters.test262)
            .add(&registry::BABEL, &BabelMetadataParser, &filters.babel)
            .add(&registry::TYPESCRIPT, &TypeScriptMetadataParser, &filters.typescript)
            .add(&registry::MISC, &MiscMetadataParser, &filters.misc)
            .build();

        self.run_suites(&CodegenRunner, suites);
    }

    /// Formatter conformance tests
    pub fn run_formatter(&self) {
        let filters = StandardFilters::new();

        let suites = ToolSuites::new("formatter")
            .add(&registry::TEST262, &Test262MetadataParser, &filters.test262)
            .add(&registry::BABEL, &BabelMetadataParser, &filters.babel)
            .add(&registry::TYPESCRIPT, &TypeScriptMetadataParser, &filters.typescript)
            .add(&registry::MISC, &MiscMetadataParser, &filters.misc)
            .build();

        self.run_suites(&FormatterRunner, suites);
    }

    /// Transformer conformance tests
    pub fn run_transformer(&self) {
        let filters = StandardFilters::new();

        let suites = ToolSuites::new("transformer")
            .add(&registry::TEST262, &Test262MetadataParser, &filters.test262)
            .add(&registry::BABEL, &BabelMetadataParser, &filters.babel)
            .add(&registry::TYPESCRIPT, &TypeScriptMetadataParser, &filters.typescript)
            .add(&registry::MISC, &MiscMetadataParser, &filters.misc)
            .build();

        self.run_suites(&TransformerRunner, suites);
    }

    /// Minifier conformance tests
    pub fn run_minifier(&self) {
        // Minifier filters have special handling (NoStrict, TypeScript exclusions)
        let test262_filter = MinifierTest262Filter::new();
        let babel_filter = MinifierBabelFilter::new();

        let suites = ToolSuites::new("minifier")
            .add(&registry::TEST262, &Test262MetadataParser, &test262_filter)
            .add(&registry::BABEL, &BabelMetadataParser, &babel_filter)
            .build();

        self.run_suites(&MinifierRunner, suites);
    }

    /// Transpiler conformance tests
    pub fn run_transpiler(&self) {
        use crate::typescript::{TranspileFilter, TranspileRunner, TranspileValidator};

        let filter = TranspileFilter::new();
        let validator = TranspileValidator;
        let reporter = self.reporter();

        // Transpiler uses special naming ("transpile" not "transpiler_transpile")
        let root_path = PathBuf::from(registry::TYPESCRIPT_TRANSPILE.root_path);
        let paths = FileDiscovery::new(root_path).discover_paths(
            "transpile",
            self.filter.as_deref(),
            &|path| filter.skip_path(path),
        );

        let results = pipeline::run_from_paths(
            &paths,
            &TypeScriptMetadataParser,
            &filter,
            &TranspileRunner,
            &validator,
        );

        reporter.report(
            "transpile",
            &results,
            Path::new(registry::TYPESCRIPT_TRANSPILE.display_path),
        );
    }

    /// ESTree conformance tests
    pub fn run_estree(&self) {
        let validator = EstreeComparisonValidator::new();
        let reporter = self.reporter();

        // Test262 - Acorn comparison
        let test262_runner = EstreeTest262Runner;
        let test262_loader = EstreeTest262Loader::new(Test262MetadataParser);
        let test262_filter = EstreeTest262Filter::new();
        let test262_paths = FileDiscovery::new(PathBuf::from("test262/test")).discover_paths(
            "test262",
            self.filter.as_deref(),
            &|path| test262_filter.skip_path(path),
        );
        let test262_results = Self::run_estree_from_paths(
            &test262_paths,
            &test262_loader,
            &test262_filter,
            &test262_runner,
            &validator,
        );
        reporter.report("estree_test262", &test262_results, Path::new("test262/test"));

        // JSX - Acorn JSX comparison
        let jsx_runner = EstreeJsxRunner;
        let jsx_loader = EstreeJsxLoader::new();
        let jsx_filter = MiscFilter::new(); // No special filtering for JSX
        let jsx_paths = FileDiscovery::new(PathBuf::from("acorn-test262/tests/acorn-jsx"))
            .discover_paths("estree_acorn_jsx", self.filter.as_deref(), &|p| {
                p.extension().and_then(|e| e.to_str()) != Some("jsx")
            });
        let jsx_results = Self::run_estree_from_paths(
            &jsx_paths,
            &jsx_loader,
            &jsx_filter,
            &jsx_runner,
            &validator,
        );
        reporter.report(
            "estree_acorn_jsx",
            &jsx_results,
            Path::new("acorn-test262/tests/acorn-jsx"),
        );

        // TypeScript - TS-ESLint comparison
        let typescript_runner = EstreeTypescriptRunner;
        let typescript_loader = EstreeTypescriptLoader::new(TypeScriptMetadataParser);
        let typescript_filter = EstreeTypescriptFilter::new();
        let typescript_paths = FileDiscovery::new(PathBuf::from("typescript/tests/cases"))
            .discover_paths("typescript", self.filter.as_deref(), &|path| {
                typescript_filter.skip_path(path)
            });
        let typescript_results = Self::run_estree_from_paths(
            &typescript_paths,
            &typescript_loader,
            &typescript_filter,
            &typescript_runner,
            &validator,
        );
        reporter.report(
            "estree_typescript",
            &typescript_results,
            Path::new("typescript/tests/cases"),
        );
    }

    /// Run ESTree pipeline with TestLoader support
    fn run_estree_from_paths(
        paths: &[PathBuf],
        loader: &dyn suite::TestLoader,
        filter: &dyn suite::TestFilter,
        runner: &dyn suite::TestRunner,
        validator: &dyn suite::ResultValidator,
    ) -> Vec<ExecutedTest> {
        use crate::suite::{ParsedTest, TestDescriptor};
        use rayon::prelude::*;

        paths
            .par_iter()
            .filter_map(|path| {
                // Load: Read file + metadata + expected output
                let descriptor = TestDescriptor::FilePath(path.clone());
                let test = loader.load(&descriptor)?;

                // Apply test-level filtering
                let parsed_test = ParsedTest {
                    path: path.clone(),
                    code: test.code.clone(),
                    source_type: test.source_type,
                    should_fail: test.should_fail,
                    metadata: test.metadata.clone(),
                };
                if filter.skip_test(&parsed_test) {
                    return None;
                }

                // Execute: Run the tool
                let result = runner.execute_sync(&test)?;

                // Validate: Compare result with expectations
                let test_result = validator.validate(&test, result);

                Some(ExecutedTest {
                    path: path.clone(),
                    should_fail: test.should_fail,
                    result: test_result,
                })
            })
            .collect()
    }

    /// Runtime conformance tests with async execution
    /// # Panics
    pub fn run_runtime(&self) {
        use std::process::Command;

        // Start Node.js runtime server
        let runtime_path =
            workspace_root().join("src/runtime/runtime.js").to_string_lossy().to_string();
        let mut runtime_process = Command::new("node")
            .args(["--experimental-vm-modules", &runtime_path])
            .spawn()
            .expect("Run runtime.js failed");

        // Give server time to start
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Run async pipeline
        let runner = crate::runtime::RuntimeRunner;
        let filter = crate::runtime::RuntimeFilter::new();
        let parser = crate::test262::Test262MetadataParser;

        // Get Test262 paths with runtime-specific filtering
        let paths = FileDiscovery::new(PathBuf::from("test262/test")).discover_paths(
            "test262",
            self.filter.as_deref(),
            &|path| filter.skip_path(path),
        );

        // Run async pipeline using tokio runtime
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let results = runtime.block_on(async {
            Self::run_async_from_paths(&paths, &parser, &filter, &runner).await
        });

        self.reporter().report("runtime_test262", &results, Path::new("test262/test"));

        // Cleanup: Stop runtime server
        let _ = runtime_process.wait();
        let _ = runtime_process.kill();
    }

    /// NodeCompat conformance tests
    pub fn run_nodecompat(&self) {
        use crate::suite::TestSource;

        let source = NodeCompatSource::new();
        let loader = NodeCompatLoader::new();
        let filter = NodeCompatFilter::new();
        let runner = NodeCompatRunner;

        // Get test descriptors from JSON source
        let descriptors = source.discover(self.filter.as_deref());

        // Run pipeline using TestSource
        let results = Self::run_from_test_source(&descriptors, &loader, &filter, &runner);

        // Print and save results
        self.reporter().report("minifier_node_compat", &results, Path::new("node-compat-table"));
    }

    /// Run pipeline with TestSource (for tests not from filesystem)
    fn run_from_test_source(
        descriptors: &[suite::TestDescriptor],
        loader: &dyn suite::TestLoader,
        filter: &dyn suite::TestFilter,
        runner: &dyn suite::TestRunner,
    ) -> Vec<ExecutedTest> {
        use rayon::prelude::*;

        descriptors
            .par_iter()
            .filter_map(|descriptor| {
                // Load test data
                let loaded_test = loader.load(descriptor)?;

                // Create ParsedTest for filtering
                let parsed_test = suite::ParsedTest {
                    path: PathBuf::from(&loaded_test.id),
                    code: loaded_test.code.clone(),
                    source_type: loaded_test.source_type,
                    should_fail: loaded_test.should_fail,
                    metadata: loaded_test.metadata.clone(),
                };

                // Apply filter
                if filter.skip_test(&parsed_test) {
                    return None;
                }

                // Execute test
                let exec_result = runner.execute_sync(&loaded_test)?;

                // Convert ErrorKind to TestResult
                let result = match exec_result.error_kind {
                    suite::ErrorKind::None => TestResult::Passed,
                    suite::ErrorKind::Errors(errors) => {
                        let error_msg = errors.join("\n");
                        // Use ParseError for minifier errors (matches original behavior)
                        TestResult::ParseError(error_msg, exec_result.panicked)
                    }
                    suite::ErrorKind::Mismatch { case, actual, expected } => {
                        TestResult::Mismatch(case, actual, expected)
                    }
                    suite::ErrorKind::Generic { case, error } => {
                        TestResult::GenericError(case, error)
                    }
                };

                Some(ExecutedTest {
                    path: PathBuf::from(&loaded_test.id),
                    should_fail: loaded_test.should_fail,
                    result,
                })
            })
            .collect()
    }

    /// Run async pipeline with TestRunner
    async fn run_async_from_paths(
        paths: &[PathBuf],
        parser: &dyn suite::MetadataParser,
        filter: &dyn suite::TestFilter,
        runner: &dyn suite::TestRunner,
    ) -> Vec<ExecutedTest> {
        use crate::discovery::FileDiscovery;
        use futures::stream::{FuturesUnordered, StreamExt};

        // Process tests in batches for better async performance
        const BATCH_SIZE: usize = 100;

        let mut all_results = Vec::new();

        for chunk in paths.chunks(BATCH_SIZE) {
            let mut futures = FuturesUnordered::new();

            for path in chunk {
                // Read file
                let Ok(code) = FileDiscovery::read_file(path) else {
                    continue; // Skip files that can't be read
                };

                // Parse metadata
                let metadata = parser.parse(path, &code);

                // Create ParsedTest for filtering
                let parsed_test = suite::ParsedTest {
                    path: path.clone(),
                    code: code.clone(),
                    metadata: metadata.clone(),
                    source_type: metadata.determine_source_type(path),
                    should_fail: metadata.should_fail(),
                };

                // Skip filtered tests
                if filter.skip_test(&parsed_test) {
                    continue;
                }

                // Load test
                let test = suite::LoadedTest {
                    id: path.to_string_lossy().into_owned(),
                    code,
                    metadata,
                    source_type: parsed_test.source_type,
                    should_fail: parsed_test.should_fail,
                    expected: suite::ExpectedOutput::None,
                };

                // Execute async
                let path_clone = path.clone();
                let should_fail = test.should_fail;
                let future = async move {
                    let result = runner.execute_async(&test).await?;

                    // Convert ErrorKind to TestResult
                    let test_result = match result.error_kind {
                        suite::ErrorKind::None => {
                            if should_fail {
                                TestResult::ParseError(
                                    "Expected an error but test passed".to_string(),
                                    false,
                                )
                            } else {
                                TestResult::Passed
                            }
                        }
                        suite::ErrorKind::Errors(errors) => {
                            let error_msg = errors.join("\n");
                            if should_fail {
                                TestResult::CorrectError(error_msg, result.panicked)
                            } else {
                                TestResult::ParseError(error_msg, result.panicked)
                            }
                        }
                        suite::ErrorKind::Mismatch { case, actual, expected } => {
                            TestResult::Mismatch(case, actual, expected)
                        }
                        suite::ErrorKind::Generic { case, error } => {
                            TestResult::GenericError(case, error)
                        }
                    };

                    Some(ExecutedTest { path: path_clone, should_fail, result: test_result })
                };

                futures.push(future);
            }

            // Collect results from this batch
            while let Some(result) = futures.next().await {
                if let Some(executed_test) = result {
                    all_results.push(executed_test);
                }
            }
        }

        all_results
    }
}

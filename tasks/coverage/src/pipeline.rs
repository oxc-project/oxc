//! Test Pipeline - Unified ETL execution for all test suites
//!
//! This module provides a composable pipeline that handles the common pattern:
//! Load → Filter → Execute → Validate

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::{
    discovery::FileDiscovery,
    misc::SyntheticTest,
    suite::{
        ExecutedTest, ExecutionResult, ExpectedOutput, LoadedTest, MetadataParser, ParsedTest,
        ResultValidator, TestFilter, TestMetadata, TestResult, TestRunner,
    },
};

/// A loaded test ready for filtering and execution
pub struct TestInput {
    pub path: PathBuf,
    pub code: String,
    pub metadata: TestMetadata,
    pub source_type: oxc::span::SourceType,
    pub should_fail: bool,
    pub expected: ExpectedOutput,
}

impl TestInput {
    /// Create from file path using metadata parser
    pub fn from_path(path: &Path, parser: &dyn MetadataParser) -> Option<Self> {
        let code = FileDiscovery::read_file(path).ok()?;
        let metadata = parser.parse(path, &code);
        let source_type = metadata.determine_source_type(path);
        let should_fail = match &metadata {
            TestMetadata::Misc => path.to_string_lossy().contains("fail"),
            _ => metadata.should_fail(),
        };

        Some(Self {
            path: path.to_path_buf(),
            code,
            metadata,
            source_type,
            should_fail,
            expected: ExpectedOutput::None,
        })
    }

    /// Create from synthetic test
    pub fn from_synthetic(test: &SyntheticTest, parser: &dyn MetadataParser) -> Self {
        let metadata = parser.parse(&test.path, &test.code);
        let source_type = metadata.determine_source_type(&test.path);
        let should_fail = match &metadata {
            TestMetadata::Misc => test.path.to_string_lossy().contains("fail"),
            _ => metadata.should_fail(),
        };

        Self {
            path: test.path.clone(),
            code: test.code.clone(),
            metadata,
            source_type,
            should_fail,
            expected: ExpectedOutput::None,
        }
    }

    /// Convert to ParsedTest for filtering
    fn to_parsed_test(&self) -> ParsedTest {
        ParsedTest {
            path: self.path.clone(),
            code: self.code.clone(),
            source_type: self.source_type,
            should_fail: self.should_fail,
            metadata: self.metadata.clone(),
        }
    }

    /// Convert to LoadedTest for execution
    fn to_loaded_test(&self) -> LoadedTest {
        LoadedTest {
            id: self.path.to_string_lossy().into_owned(),
            code: self.code.clone(),
            metadata: self.metadata.clone(),
            source_type: self.source_type,
            should_fail: self.should_fail,
            expected: self.expected.clone(),
        }
    }
}

/// Run the standard test pipeline: Filter → Execute → Validate
///
/// This is the unified entry point for all sync test execution.
pub fn run_pipeline<'a, I>(
    inputs: I,
    filter: &dyn TestFilter,
    runner: &dyn TestRunner,
    validator: &dyn ResultValidator,
) -> Vec<ExecutedTest>
where
    I: IntoParallelIterator<Item = TestInput> + 'a,
{
    inputs
        .into_par_iter()
        .filter_map(|input| {
            // Filter
            if filter.skip_test(&input.to_parsed_test()) {
                return None;
            }

            let loaded_test = input.to_loaded_test();

            // Execute
            let result = runner.execute_sync(&loaded_test)?;

            // Validate
            let test_result = validator.validate(&loaded_test, result);

            Some(ExecutedTest {
                path: input.path,
                should_fail: loaded_test.should_fail,
                result: test_result,
            })
        })
        .collect()
}

/// Run pipeline from file paths using a metadata parser
pub fn run_from_paths(
    paths: &[PathBuf],
    parser: &dyn MetadataParser,
    filter: &dyn TestFilter,
    runner: &dyn TestRunner,
    validator: &dyn ResultValidator,
) -> Vec<ExecutedTest> {
    let inputs: Vec<_> =
        paths.par_iter().filter_map(|path| TestInput::from_path(path, parser)).collect();

    run_pipeline(inputs, filter, runner, validator)
}

/// Run pipeline from synthetic (in-memory) tests
pub fn run_from_synthetic(
    tests: &[SyntheticTest],
    parser: &dyn MetadataParser,
    filter: &dyn TestFilter,
    runner: &dyn TestRunner,
    validator: &dyn ResultValidator,
) -> Vec<ExecutedTest> {
    let inputs: Vec<_> =
        tests.par_iter().map(|test| TestInput::from_synthetic(test, parser)).collect();

    run_pipeline(inputs, filter, runner, validator)
}

/// Run pipeline with custom validation logic (for node_compat)
///
/// This variant doesn't use a ResultValidator and instead applies custom
/// result conversion logic.
#[expect(dead_code)]
pub fn run_with_custom_validation<F>(
    inputs: Vec<TestInput>,
    filter: &dyn TestFilter,
    runner: &dyn TestRunner,
    result_converter: F,
) -> Vec<ExecutedTest>
where
    F: Fn(ExecutionResult, bool) -> TestResult + Sync,
{
    inputs
        .into_par_iter()
        .filter_map(|input| {
            // Filter
            if filter.skip_test(&input.to_parsed_test()) {
                return None;
            }

            let loaded_test = input.to_loaded_test();
            let should_fail = loaded_test.should_fail;

            // Execute
            let result = runner.execute_sync(&loaded_test)?;

            // Custom validation
            let test_result = result_converter(result, should_fail);

            Some(ExecutedTest { path: input.path, should_fail, result: test_result })
        })
        .collect()
}

mod meta;

use std::path::{Path, PathBuf};

use oxc_span::SourceType;

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

use self::meta::TestCaseContent;

const TESTS_ROOT: &str = "tasks/coverage/typescript/tests/";

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(TESTS_ROOT).join("cases"), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TypeScriptSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        // stack overflows in compiler tests
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));
        let unsupported_tests = [
            // these 2 relies on the ts "target" option
            "functionWithUseStrictAndSimpleParameterList.ts",
            "parameterInitializerBeforeDestructuringEmit.ts",
        ]
        .iter()
        .any(|p| path.to_string_lossy().contains(p));
        !supported_paths || unsupported_tests
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }
    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct TypeScriptCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    result: TestResult,
    meta: TestCaseContent,
}

impl TypeScriptCase {
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }

    pub fn meta(&self) -> &TestCaseContent {
        &self.meta
    }
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let meta = TestCaseContent::make_units_from_test(&path, &code);
        let compiler_options = &meta.settings;
        let is_module = ["esnext", "es2022", "es2020", "es2015"]
            .into_iter()
            .any(|module| compiler_options.modules.contains(&module.to_string()));
        let source_type = SourceType::from_path(&path)
            .unwrap()
            .with_script(true)
            .with_module(is_module)
            .with_jsx(!compiler_options.jsx.is_empty())
            .with_typescript_definition(compiler_options.declaration);
        Self {
            path,
            // FIXME: current skip multi-file test cases, if doesn't skip in the future, need to handle multi-file test cases
            // Use meta.tests[0].content.clone() instead of code to get without meta options code
            code: meta.tests[0].content.clone(),
            source_type,
            result: TestResult::ToBeRun,
            meta,
        }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        !self.meta.error_files.is_empty()
    }

    fn skip_test_case(&self) -> bool {
        // skip multi-file test cases for now
        self.meta.tests.len() > 1
    }

    fn run(&mut self) {
        self.result = self.execute(self.source_type);
    }
}

use std::path::{Path, PathBuf};

use oxc::span::SourceType;

use crate::suite::{Case, Suite, TestResult};

const FIXTURES_PATH: &str = "misc";

pub struct MiscSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> MiscSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(FIXTURES_PATH), test_cases: vec![] }
    }

    fn extra_cases() -> Vec<T> {
        vec![Self::huge_binary_expression(), Self::huge_nested_statements()]
    }

    fn huge_binary_expression() -> T {
        let code = String::from("a") + &"+ a".repeat(1000);
        T::new(PathBuf::from("huge_binary_expression.js"), code.to_string())
    }

    fn huge_nested_statements() -> T {
        let take = 1000;
        let code = "if (true) {".repeat(take) + &"}".repeat(take);
        T::new(PathBuf::from("huge_nested_statements.js"), code.to_string())
    }
}

impl<T: Case> Suite<T> for MiscSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases;
    }

    fn save_extra_test_cases(&mut self) {
        self.test_cases.extend(Self::extra_cases());
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct MiscCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    should_fail: bool,
    result: TestResult,
}

impl MiscCase {
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }
}

impl Case for MiscCase {
    fn new(path: PathBuf, code: String) -> Self {
        let should_fail = path.to_string_lossy().contains("fail");
        let source_type = SourceType::from_path(&path).unwrap();
        Self { path, code, source_type, should_fail, result: TestResult::ToBeRun }
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
        self.should_fail
    }

    fn run(&mut self) {
        let result = self.execute(self.source_type);
        self.set_result(result);
    }
}

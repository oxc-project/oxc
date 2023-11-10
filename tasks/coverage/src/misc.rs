use std::path::{Path, PathBuf};

use oxc_span::SourceType;

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

const FIXTURES_PATH: &str = "tasks/coverage/misc";

pub struct MiscSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> MiscSuite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for MiscSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
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

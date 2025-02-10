mod meta;

use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use oxc::span::SourceType;

pub use self::meta::{MetaData, Phase, TestFlag};
use crate::suite::{Case, Suite, TestResult};

const FIXTURES_PATH: &str = "test262/test";

pub struct Test262Suite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> Test262Suite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for Test262Suite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        let path = path.cow_replace('\\', "/");
        path.contains("test262/test/staging") ||
        // ignore markdown files
        path.ends_with(".md") ||
        // ignore fixtures
        path.contains("_FIXTURE")
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct Test262Case {
    path: PathBuf,
    code: String,
    meta: MetaData,
    should_fail: bool,
    always_strict: bool,
    result: TestResult,
}

impl Test262Case {
    /// # Panics
    pub fn read_metadata(code: &str) -> MetaData {
        let (start, end) = (code.find("/*---").unwrap(), code.find("---*/").unwrap());
        let s = &code[start + 5..end].replace('\r', "\n");
        MetaData::from_str(s)
    }

    fn compute_should_fail(meta: &MetaData) -> bool {
        meta.negative.as_ref().filter(|n| n.phase == Phase::Parse).is_some()
    }

    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }

    pub fn meta(&self) -> &MetaData {
        &self.meta
    }

    pub fn is_module(&self) -> bool {
        self.meta.flags.contains(&TestFlag::Module)
    }

    pub fn is_only_strict(&self) -> bool {
        self.meta.flags.contains(&TestFlag::OnlyStrict)
    }

    pub fn is_no_strict(&self) -> bool {
        self.meta.flags.contains(&TestFlag::NoStrict)
    }

    pub fn is_raw(&self) -> bool {
        self.meta.flags.contains(&TestFlag::Raw)
    }

    pub fn is_async(&self) -> bool {
        self.meta.flags.contains(&TestFlag::Async)
    }
}

impl Case for Test262Case {
    fn new(path: PathBuf, code: String) -> Self {
        let meta = Self::read_metadata(&code);
        let should_fail = Self::compute_should_fail(&meta);
        Self { path, code, meta, should_fail, always_strict: false, result: TestResult::ToBeRun }
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

    fn always_strict(&self) -> bool {
        self.always_strict
    }

    fn skip_test_case(&self) -> bool {
        false
        // []
        // .iter()
        // .any(|feature| self.meta.features.iter().any(|f| **f == **feature))
    }

    // Unless configured otherwise (via the noStrict, onlyStrict, module, or raw flags),
    // each test must be executed twice: once in ECMAScript's non-strict mode, and again in ECMAScript's strict mode.
    // To run in strict mode, the test contents must be modified prior to execution--
    // a "use strict" directive must be inserted as the initial character sequence of the file
    // https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/INTERPRETING.md#strict-mode
    fn run(&mut self) {
        let flags = &self.meta.flags;
        let source_type = SourceType::cjs();

        self.result = if flags.contains(&TestFlag::OnlyStrict) {
            self.always_strict = true;
            self.execute(source_type)
        } else if flags.contains(&TestFlag::Module) {
            self.execute(source_type.with_module(true))
        } else if flags.contains(&TestFlag::NoStrict) || flags.contains(&TestFlag::Raw) {
            self.execute(source_type)
        } else {
            self.always_strict = true;
            let res = self.execute(source_type);
            if matches!(res, TestResult::Passed) {
                self.always_strict = false;
                self.execute(source_type)
            } else {
                res
            }
        };
    }
}

use std::path::{Path, PathBuf};

use oxc::span::SourceType;

use crate::{
    Driver,
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
};

/// Check if a string contains any non-ASCII characters (bytes > 0x7F).
fn contains_non_ascii(s: &str) -> bool {
    s.bytes().any(|b| b > 0x7F)
}

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let result = Driver { codegen: true, ..Driver::default() }.idempotency(
        "Normal",
        source_text,
        source_type,
    );
    if result != TestResult::Passed {
        return result;
    }

    let result = Driver { codegen: true, remove_whitespace: true, ..Driver::default() }
        .idempotency("Minify", source_text, source_type);
    if result != TestResult::Passed {
        return result;
    }

    // Test ascii_only mode: output must not contain any non-ASCII characters.
    // Skip this test if the source contains non-ASCII, since comments are preserved
    // as-is and may contain non-ASCII characters (e.g., author names).
    if !contains_non_ascii(source_text) {
        let mut driver = Driver { codegen: true, ascii_only: true, ..Driver::default() };
        driver.run(source_text, source_type);
        if contains_non_ascii(&driver.printed) {
            return TestResult::GenericError(
                "AsciiOnly",
                format!("Output contains non-ASCII characters: {:?}", driver.printed),
            );
        }
    }

    TestResult::Passed
}

pub struct CodegenTest262Case {
    base: Test262Case,
}

impl Case for CodegenTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.should_fail() || self.base.skip_test_case()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct CodegenBabelCase {
    base: BabelCase,
}

impl Case for CodegenBabelCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: BabelCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct CodegenTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for CodegenTypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: TypeScriptCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let units = self.base.units.clone();
        for unit in units {
            let result = get_result(&unit.content, unit.source_type);
            if result != TestResult::Passed {
                self.base.result = result;
                return;
            }
        }
        self.base.result = TestResult::Passed;
    }
}

pub struct CodegenMiscCase {
    base: MiscCase,
}

impl Case for CodegenMiscCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: MiscCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

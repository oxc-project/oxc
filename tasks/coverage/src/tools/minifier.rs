use std::path::{Path, PathBuf};

use oxc::span::SourceType;

use crate::{
    babel::BabelCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    Driver,
};

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    Driver { compress: true, codegen: true, ..Driver::default() }.idempotency(
        "Compress",
        source_text,
        source_type,
    )
}

pub struct MinifierTest262Case {
    base: Test262Case,
}

impl Case for MinifierTest262Case {
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
            // Unable to minify non-strict code, which may contain syntaxes that the minifier do not support (e.g. `with`).
            || self.base.is_no_strict()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct MinifierBabelCase {
    base: BabelCase,
}

impl Case for MinifierBabelCase {
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
        self.base.skip_test_case()
            || self.base.should_fail()
            || self.base.source_type().is_typescript()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

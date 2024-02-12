use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    typescript::TypeScriptCase,
};

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let options = PrettierOptions::default();

    let allocator = Allocator::default();
    let ParserReturn { program, trivias, .. } =
        Parser::new(&allocator, source_text, source_type).preserve_parens(false).parse();
    let source_text1 = Prettier::new(&allocator, source_text, trivias, options).build(&program);

    let allocator = Allocator::default();
    let ParserReturn { program, trivias, .. } =
        Parser::new(&allocator, &source_text1, source_type).preserve_parens(false).parse();
    let source_text2 = Prettier::new(&allocator, &source_text1, trivias, options).build(&program);

    if source_text1 == source_text2 {
        TestResult::Passed
    } else {
        TestResult::ParseError(String::new(), false)
    }
}

pub struct PrettierTest262Case {
    base: Test262Case,
}

impl Case for PrettierTest262Case {
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
        self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct PrettierBabelCase {
    base: BabelCase,
}

impl Case for PrettierBabelCase {
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

pub struct PrettierTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for PrettierTypeScriptCase {
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
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct PrettierMiscCase {
    base: MiscCase,
}

impl Case for PrettierMiscCase {
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

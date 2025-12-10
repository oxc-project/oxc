use std::path::{Path, PathBuf};

use oxc::{
    allocator::Allocator,
    parser::{Parser, ParserReturn},
    span::SourceType,
};
use oxc_formatter::{FormatOptions, Formatter, get_parse_options};

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
};

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let options = FormatOptions::default();

    let allocator = Allocator::default();
    let ParserReturn { program, errors, .. } =
        Parser::new(&allocator, source_text, source_type).with_options(get_parse_options()).parse();

    if !errors.is_empty() {
        // Skip test if input source code has parse errors
        return TestResult::Passed;
    }

    let source_text1 = Formatter::new(&allocator, options.clone()).build(&program);

    let allocator = Allocator::default();
    let ParserReturn { program, errors, .. } = Parser::new(&allocator, &source_text1, source_type)
        .with_options(get_parse_options())
        .parse();

    if !errors.is_empty() {
        return TestResult::ParseError(
            errors.iter().map(std::string::ToString::to_string).collect(),
            false,
        );
    }

    let source_text2 = Formatter::new(&allocator, options).build(&program);

    if source_text1 == source_text2 {
        TestResult::Passed
    } else {
        TestResult::Mismatch("Mismatch", source_text1, source_text2)
    }
}

pub struct FormatterTest262Case {
    base: Test262Case,
}

impl Case for FormatterTest262Case {
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

pub struct FormatterBabelCase {
    base: BabelCase,
}

impl Case for FormatterBabelCase {
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

pub struct FormatterTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for FormatterTypeScriptCase {
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
            self.base.code.clone_from(&unit.content);
            let result = get_result(&unit.content, unit.source_type);
            if result != TestResult::Passed {
                self.base.result = result;
                return;
            }
        }
        self.base.result = TestResult::Passed;
    }
}

pub struct FormatterMiscCase {
    base: MiscCase,
}

impl Case for FormatterMiscCase {
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

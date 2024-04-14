use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    typescript::TypeScriptCase,
};

/// Runs the transformer and make sure it doesn't crash.
/// TODO: add codegen to turn on idempotency test.
fn get_result(source_text: &str, source_type: SourceType, source_path: &Path) -> TestResult {
    let allocator = Allocator::default();

    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();

    let semantic_ret = SemanticBuilder::new(source_text, source_type)
        .with_trivias(parser_ret.trivias)
        .with_check_syntax_error(true)
        .build(&parser_ret.program);

    let mut program = parser_ret.program;
    let options = TransformOptions::default();
    let _ = Transformer::new(&allocator, source_path, semantic_ret.semantic, options)
        .build(&mut program);
    TestResult::Passed
}

pub struct TransformerTest262Case {
    base: Test262Case,
}

impl Case for TransformerTest262Case {
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
        let result = get_result(source_text, source_type, self.path());
        self.base.set_result(result);
    }
}

pub struct TransformerBabelCase {
    base: BabelCase,
}

impl Case for TransformerBabelCase {
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
        let result = get_result(source_text, source_type, self.path());
        self.base.set_result(result);
    }
}

pub struct TransformerTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for TransformerTypeScriptCase {
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
        let result = get_result(self.base.code(), self.base.source_type(), self.path());
        self.base.set_result(result);
    }
}

pub struct TransformerMiscCase {
    base: MiscCase,
}

impl Case for TransformerMiscCase {
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
        let result = get_result(self.base.code(), self.base.source_type(), self.path());
        self.base.set_result(result);
    }
}

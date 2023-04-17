use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::SemanticBuilder;

use crate::babel::BabelCase;
use crate::suite::{Case, TestResult};
use crate::test262::{Test262Case, TestFlag};

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
// Test minification by minifying twice because it is a idempotent
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let allocator = Allocator::default();
    let printer_options = PrinterOptions::default();
    let minifier_options = MinifierOptions::default();

    let ret1 = Parser::new(&allocator, source_text, source_type).parse();
    let program1 = allocator.alloc(ret1.program);
    Minifier::new(&allocator, minifier_options).build(program1);
    let _semantic = SemanticBuilder::new(source_text, source_type, &ret1.trivias).build(program1);
    let source_text1 = Printer::new(source_text.len(), printer_options).build(program1);

    let ret2 = Parser::new(&allocator, &source_text1, source_type).parse();
    let program2 = allocator.alloc(ret2.program);
    Minifier::new(&allocator, minifier_options).build(program2);
    let _semantic = SemanticBuilder::new(&source_text1, source_type, &ret2.trivias).build(program2);
    let source_text2 = Printer::new(source_text1.len(), printer_options).build(program2);

    if source_text1 == source_text2 {
        TestResult::Passed
    } else {
        TestResult::ParseError(String::new(), false)
    }
}

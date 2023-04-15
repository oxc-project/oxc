use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};

use crate::babel::BabelCase;
use crate::suite::{Case, TestResult};
use crate::test262::{Test262Case, TestFlag};

pub struct PrinterTest262Case {
    base: Test262Case,
}

impl Case for PrinterTest262Case {
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
        let source_type = *SourceType::default().with_module(is_module);

        // Test printer
        let printer_options = PrinterOptions::default();
        let result = get_result(source_text, source_type, printer_options);

        if !matches!(result, TestResult::Passed) {
            self.base.set_result(result);
            return;
        }

        // Test whitespace minification
        let printer_options =
            PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() };
        let result = get_result(source_text, source_type, printer_options);
        self.base.set_result(result);
    }
}

pub struct PrinterBabelCase {
    base: BabelCase,
}

impl Case for PrinterBabelCase {
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

        // Test printer
        let printer_options = PrinterOptions::default();
        let result = get_result(source_text, source_type, printer_options);

        if !matches!(result, TestResult::Passed) {
            self.base.set_result(result);
            return;
        }

        // Test whitespace minification
        let printer_options =
            PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() };
        let result = get_result(source_text, source_type, printer_options);
        self.base.set_result(result);
    }
}

fn get_result(source_text: &str, source_type: SourceType, options: PrinterOptions) -> TestResult {
    let allocator = Allocator::default();
    let program1 = Parser::new(&allocator, source_text, source_type).parse().program;
    let source_text1 = Printer::new(source_text.len(), options).build(&program1);
    let program2 = Parser::new(&allocator, &source_text1, source_type).parse().program;
    let source_text2 = Printer::new(source_text1.len(), options).build(&program2);
    if source_text1 == source_text2 {
        TestResult::Passed
    } else {
        TestResult::ParseError(String::new(), false)
    }
}

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_common::PaddedStringView;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};

use crate::suite::{Case, TestResult};
use crate::test262::{Test262Case, TestFlag};

pub struct PrinterTest262Case {
    base: Test262Case,
}

impl Case for PrinterTest262Case {
    fn new(path: PathBuf, code: PaddedStringView) -> Self {
        Self { base: Test262Case::new(path, code) }
    }

    fn code(&self) -> &PaddedStringView {
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
        // Test printer
        let printer_options = PrinterOptions::default();
        let result = self.get_result(printer_options);

        if !matches!(result, TestResult::Passed) {
            self.base.set_result(result);
            return;
        }

        // Test whitespace minification
        let printer_options =
            PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() };
        let result = self.get_result(printer_options);
        self.base.set_result(result);
    }
}

impl PrinterTest262Case {
    fn get_result(&self, options: PrinterOptions) -> TestResult {
        let allocator = Allocator::default();
        let source_text = self.base.code();
        let source_type = {
            let mut builder = SourceType::builder();
            if self.base.meta().flags.contains(&TestFlag::Module) {
                builder = builder.module();
            }
            builder.build()
        };
        let program1 = Parser::new(&allocator, &source_text, source_type).parse().program;
        let printed_text1 = Printer::new(source_text.len(), options).build(&program1);
        let source_text1 = (&printed_text1).into();
        let program2 = Parser::new(&allocator, &source_text1, source_type).parse().program;
        let source_text2 = Printer::new(printed_text1.len(), options).build(&program2);
        if printed_text1 == source_text2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch(printed_text1.to_string(), source_text2)
        }
    }
}

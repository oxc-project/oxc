use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    typescript::TypeScriptCase,
};

fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let normal_result = get_normal_result(source_text, source_type);
    if !normal_result {
        return TestResult::CodegenError("Normal");
    };

    let minify_result = get_minify_result(source_text, source_type);
    if !minify_result {
        return TestResult::CodegenError("Minify");
    }

    TestResult::Passed
}

/// Idempotency test
fn get_normal_result(source_text: &str, source_type: SourceType) -> bool {
    let options = CodegenOptions::default();
    let allocator = Allocator::default();
    let source_text1 = {
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        Codegen::<false>::new("", source_text, ret.trivias, options).build(&ret.program).source_text
    };

    let source_text2 = {
        let ret = Parser::new(&allocator, &source_text1, source_type).parse();
        Codegen::<false>::new("", &source_text1, ret.trivias, options)
            .build(&ret.program)
            .source_text
    };

    source_text1 == source_text2
}

/// Minify idempotency test
fn get_minify_result(source_text: &str, source_type: SourceType) -> bool {
    let options = CodegenOptions::default();
    let allocator = Allocator::default();
    let parse_result1 = Parser::new(&allocator, source_text, source_type).parse();
    let source_text1 =
        Codegen::<true>::new("", source_text, parse_result1.trivias.clone(), options)
            .build(&parse_result1.program)
            .source_text;
    let parse_result2 = Parser::new(&allocator, source_text1.as_str(), source_type).parse();
    let source_text2 = Codegen::<true>::new("", &source_text1, parse_result2.trivias, options)
        .build(&parse_result2.program)
        .source_text;
    source_text1 == source_text2
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
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
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

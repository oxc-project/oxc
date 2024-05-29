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

fn get_result(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
) -> TestResult {
    let normal_result = get_normal_result(case_name, file_name, source_text, source_type);
    let minify_result = get_minify_result(case_name, file_name, source_text, source_type);
    let typescript_result = get_typescript_result(case_name, file_name, source_text, source_type);

    if !normal_result {
        return TestResult::CodegenError("Default");
    }
    if !minify_result {
        return TestResult::CodegenError("Minify");
    }
    if !typescript_result {
        return TestResult::CodegenError("Typescript");
    }

    TestResult::Passed
}

#[allow(clippy::too_many_arguments)]
fn write_failure(
    case_name: &str,
    file_name: &Path,
    result_type: &str,
    original: &str,
    parser_result1: &str,
    source_text1: &str,
    parser_result2: &str,
    source_text2: &str,
) {
    let base_path = Path::new(&format!("./tasks/coverage/failures/{case_name}"))
        .join(file_name)
        .join(result_type);
    let _ = std::fs::create_dir_all(&base_path);
    std::fs::write(base_path.join("original.ts"), original).expect("Error writing original.ts");
    std::fs::write(base_path.join("parser_result1.txt"), parser_result1)
        .expect("Error writing parser_result1.json");
    std::fs::write(base_path.join("source_text1.ts"), source_text1)
        .expect("Error writing source_text1.ts");
    std::fs::write(base_path.join("parser_result2.txt"), parser_result2)
        .expect("Error writing parser_result2.json");
    std::fs::write(base_path.join("source_text2.ts"), source_text2)
        .expect("Error writing source_text2.ts");
}

/// Idempotency test
fn get_normal_result(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
) -> bool {
    let options = CodegenOptions::default();
    let allocator = Allocator::default();
    let parse_result1 = Parser::new(&allocator, source_text, source_type).parse();
    let source_text1 =
        Codegen::<false>::new(source_text.len(), options, None).build(&parse_result1.program);
    let parse_result2 = Parser::new(&allocator, &source_text1, source_type).parse();
    let source_text2 =
        Codegen::<false>::new(source_text1.len(), options, None).build(&parse_result2.program);
    let result = source_text1 == source_text2;

    if !result {
        let parse_result1 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result1.panicked, parse_result1.errors, parse_result1.program
        );
        let parse_result2 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result2.panicked, parse_result2.errors, parse_result2.program
        );
        write_failure(
            case_name,
            file_name,
            "normal",
            source_text,
            &parse_result1,
            &source_text1,
            &parse_result2,
            &source_text2,
        );
    }

    result
}

/// Minify idempotency test
fn get_minify_result(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
) -> bool {
    let options = CodegenOptions::default();
    let allocator = Allocator::default();
    let parse_result1 = Parser::new(&allocator, source_text, source_type).parse();
    let source_text1 =
        Codegen::<true>::new(source_text.len(), options, None).build(&parse_result1.program);
    let parse_result2 = Parser::new(&allocator, source_text1.as_str(), source_type).parse();
    let source_text2 =
        Codegen::<true>::new(source_text1.len(), options, None).build(&parse_result2.program);
    let result = source_text1 == source_text2;

    if !result {
        let parse_result1 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result1.panicked, parse_result1.errors, parse_result1.program
        );
        let parse_result2 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result2.panicked, parse_result2.errors, parse_result2.program
        );
        write_failure(
            case_name,
            file_name,
            "minify",
            source_text,
            &parse_result1,
            &source_text1,
            &parse_result2,
            &source_text2,
        );
    }

    result
}

/// TypeScript idempotency test
fn get_typescript_result(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
) -> bool {
    let options = CodegenOptions { enable_typescript: true, ..Default::default() };
    let allocator = Allocator::default();
    let parse_result1 = Parser::new(&allocator, source_text, source_type).parse();
    let source_text1 =
        Codegen::<false>::new(source_text.len(), options, None).build(&parse_result1.program);
    let parse_result2 = Parser::new(&allocator, &source_text1, source_type).parse();
    let source_text2 =
        Codegen::<false>::new(source_text1.len(), options, None).build(&parse_result2.program);
    let result = source_text1 == source_text2;

    if !result {
        let parse_result1 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result1.panicked, parse_result1.errors, parse_result1.program
        );
        let parse_result2 = format!(
            "Panicked: {:#?}\nErrors:\n{:#?}\nProgram:\n{:#?}",
            parse_result2.panicked, parse_result2.errors, parse_result2.program
        );
        write_failure(
            case_name,
            file_name,
            "typescript",
            source_text,
            &parse_result1,
            &source_text1.clone(),
            &parse_result2,
            &source_text2,
        );
    }

    result
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
        let result = get_result("test262", self.base.path(), source_text, source_type);
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
        let result = get_result("babel", self.base.path(), source_text, source_type);
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
        let result = get_result("typescript", self.base.path(), source_text, source_type);
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
        let result = get_result("misc", self.base.path(), source_text, source_type);
        self.base.set_result(result);
    }
}

#![allow(clippy::missing_panics_doc)]
pub mod esbuild;
pub mod jsdoc;
pub mod legal_comments;
pub mod pure_comments;
pub mod tester;
pub mod ts;
pub mod unit;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn codegen(source_text: &str) -> String {
    codegen_options(source_text, &CodegenOptions::default()).code
}

pub fn codegen_options(source_text: &str, options: &CodegenOptions) -> CodegenReturn {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut options = options.clone();
    options.single_quote = true;
    CodeGenerator::new().with_options(options).build(&ret.program)
}

pub fn snapshot(name: &str, cases: &[&str]) {
    snapshot_options(name, cases, &CodegenOptions::default());
}

pub fn snapshot_options(name: &str, cases: &[&str], options: &CodegenOptions) {
    use std::fmt::Write;

    let snapshot = cases.iter().enumerate().fold(String::new(), |mut w, (i, case)| {
        let result = codegen_options(case, options).code;
        write!(w, "########## {i}\n{case}\n----------\n{result}\n",).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
        insta::assert_snapshot!(name, snapshot);
    });
}

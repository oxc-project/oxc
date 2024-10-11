#![allow(clippy::missing_panics_doc)]
pub mod esbuild;
pub mod jsdoc;
pub mod pure_comments;
pub mod tester;
pub mod ts;
pub mod unit;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions, CommentOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn codegen(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .enable_comment(&ret.program, CommentOptions { preserve_annotate_comments: true })
        .build(&ret.program)
        .code
}

pub fn snapshot(name: &str, cases: &[&str]) {
    use std::fmt::Write;

    let snapshot = cases.iter().enumerate().fold(String::new(), |mut w, (i, case)| {
        write!(w, "########## {i}\n{case}\n----------\n{}\n", codegen(case)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!(name, snapshot);
    });
}

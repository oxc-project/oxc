use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{TransformOptions, Transformer};

pub struct Tester {
    source_type: SourceType,

    options: TransformOptions,

    allocator: Allocator,
}

impl Tester {
    pub fn new(filename: &str, options: TransformOptions) -> Self {
        let source_type = SourceType::from_path(filename).unwrap();
        Self { source_type, options, allocator: Allocator::default() }
    }

    pub fn test(&self, tests: &[(&str, &str)]) {
        for (source_text, expected) in tests {
            let transformed = self.transform(source_text);
            let expected = self.codegen(expected);
            assert_eq!(transformed, expected, "{source_text}");
        }
    }

    fn transform(&self, source_text: &str) -> String {
        let program = Parser::new(&self.allocator, source_text, self.source_type).parse().program;

        let semantic = SemanticBuilder::new(source_text, self.source_type).build(&program).semantic;
        let (symbols, _scope_tree) = semantic.into_symbol_table_and_scope_tree();
        let symbols = Rc::new(RefCell::new(symbols));

        let program = self.allocator.alloc(program);
        Transformer::new(&self.allocator, self.source_type, &symbols, self.options).build(program);
        Codegen::<false>::new(source_text.len(), CodegenOptions).build(program)
    }

    fn codegen(&self, source_text: &str) -> String {
        let program = Parser::new(&self.allocator, source_text, self.source_type).parse().program;
        Codegen::<false>::new(source_text.len(), CodegenOptions).build(&program)
    }
}

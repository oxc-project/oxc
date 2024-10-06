#![allow(clippy::print_stdout)]

use oxc_allocator::Allocator;
use oxc_regular_expression::{
    visit::{RegExpAstKind, Visit},
    Parser, ParserOptions,
};
use oxc_span::GetSpan;

struct TestVisitor;

impl Visit<'_> for TestVisitor {
    fn enter_node(&mut self, kind: RegExpAstKind) {
        println!("enter_node: {:?} {kind:?}", kind.span());
    }

    fn leave_node(&mut self, kind: RegExpAstKind) {
        println!("leave_node: {:?} {kind:?}", kind.span());
    }
}

fn main() {
    let source_text = r"(https?:\/\/github\.com\/(([^\s]+)\/([^\s]+))\/([^\s]+\/)?(issues|pull)\/([0-9]+))|(([^\s]+)\/([^\s]+))?#([1-9][0-9]*)($|[\s\:\;\-\(\=])";

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source_text, ParserOptions::default());
    let pattern = parser.parse().unwrap();

    let mut visitor = TestVisitor;
    visitor.visit_pattern(&pattern);
}

#![expect(clippy::print_stdout)]
//! # Regular Expression AST Visitor Example
//!
//! This example demonstrates how to use the visitor pattern to traverse
//! regular expression ASTs and inspect different node types.
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p oxc_regular_expression --example regex_visitor
//! ```

use oxc_allocator::Allocator;
use oxc_regular_expression::{
    LiteralParser, Options,
    visit::{RegExpAstKind, Visit},
};
use oxc_span::GetSpan;

/// A test visitor that logs entering and leaving AST nodes
struct TestVisitor;

impl Visit<'_> for TestVisitor {
    fn enter_node(&mut self, kind: RegExpAstKind) {
        println!("enter_node: {:?} {kind:?}", kind.span());
    }

    fn leave_node(&mut self, kind: RegExpAstKind) {
        println!("leave_node: {:?} {kind:?}", kind.span());
    }
}

/// Demonstrate regex AST traversal using the visitor pattern
fn main() {
    let source_text = r"(https?:\/\/github\.com\/(([^\s]+)\/([^\s]+))\/([^\s]+\/)?(issues|pull)\/([0-9]+))|(([^\s]+)\/([^\s]+))?#([1-9][0-9]*)($|[\s\:\;\-\(\=])";

    let allocator = Allocator::default();
    let parser = LiteralParser::new(&allocator, source_text, None, Options::default());
    let pattern = parser.parse().unwrap();

    // Visit the regex AST and log each node
    let mut visitor = TestVisitor;
    visitor.visit_pattern(&pattern);
}

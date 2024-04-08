mod tst;
mod tst_node;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::tst::TstBuilder;

fn main() {
    let source_text = "123;";
    let source_type = SourceType::default().with_module(true);
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, &source_text, source_type).parse().program;

    dbg!("AST", &program);

    let builder = TstBuilder::from_ast(program);

    dbg!("TST", &builder);
}

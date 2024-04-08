mod tst;
mod tst_node;
mod tst_visit;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::tst::Tst;
use crate::tst_visit::NumericSeparators;

fn main() {
    let source_text = "123_456;";
    // let source_text = " {123_456;}";
    let source_type = SourceType::default().with_module(true);
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, &source_text, source_type).parse().program;

    dbg!("AST", &ast);

    let mut tst = Tst::from_ast(&allocator, ast);

    dbg!("TST (before)", &tst);

    tst.add_transformer(NumericSeparators);
    tst.transform();

    dbg!("TST (after)", &tst);
}

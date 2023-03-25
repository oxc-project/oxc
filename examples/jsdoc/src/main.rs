use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap();
    let source = std::fs::read_to_string(&path).unwrap();

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&path).unwrap();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    let program = allocator.alloc(ret.program);
    let trivias = Rc::new(ret.trivias);

    let ctx = SemanticBuilder::new(&source, source_type, &trivias).build(program);
    let jsdoc = ctx.semantic.jsdoc();

    for node in ctx.semantic.nodes().iter() {
        if let Some(jsdoc) = jsdoc.get_by_node(node) {
            println!("{jsdoc:#?}");
        }
    }
}

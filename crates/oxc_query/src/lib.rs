#![feature(let_chains)]
#![allow(clippy::redundant_pub_crate)]
mod adapter;
mod edges;
mod entrypoints;
mod properties;
mod util;
mod vertex;

pub use adapter::Adapter;
pub use vertex::Vertex;

// TODO: Uncomment on next release of trustfall
#[cfg(test)]
mod test {
    // use std::{path::Path, rc::Rc};

    // use oxc_allocator::Allocator;
    // use oxc_parser::Parser;
    // use oxc_semantic::SemanticBuilder;
    // use oxc_span::SourceType;

    // use crate::{adapter::schema, Adapter};

    // #[test]
    // fn test_invariants() {
    //     let file_path = "/apple/orange.tsx";
    //     let source_text = "const apple = 1;";

    //     let allocator = Allocator::default();
    //     let source_type = SourceType::from_path(file_path).unwrap();
    //     let ret = Parser::new(&allocator, source_text, source_type).parse();
    //     let program = allocator.alloc(ret.program);
    //     let semantic_ret = SemanticBuilder::new(source_text, source_type)
    //         .with_trivias(&ret.trivias)
    //         .build(program);

    //     let adapter = Adapter { path_components: vec![], semantic: Rc::new(semantic_ret.semantic) };
    //     check_adapter_invariants(schema(), &adapter);
    // }
}

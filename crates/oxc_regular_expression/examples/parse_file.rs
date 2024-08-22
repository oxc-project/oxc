#![allow(clippy::print_stdout)]
use std::{env, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    let source_text = Arc::new(fs::read_to_string(path).unwrap());
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();

    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !parser_ret.errors.is_empty() {
        println!("Parsing failed:");
        for error in parser_ret.errors {
            let error = error.with_source_code(Arc::clone(&source_text));
            println!("{error:?}");
        }
        return;
    }

    let program = allocator.alloc(parser_ret.program);
    let semantic_ret = SemanticBuilder::new(&source_text, source_type).build(program);
    let semantic = semantic_ret.semantic;

    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::RegExpLiteral(re) => {
                let literal = re.span.source_text(&source_text);
                let parsed = oxc_regular_expression::Parser::new(
                    &allocator,
                    literal,
                    oxc_regular_expression::ParserOptions::default()
                        .with_span_offset(re.span.start),
                )
                .parse();

                println!("🍀 {literal}");
                if let Err(error) = parsed {
                    let error = error.with_source_code(Arc::clone(&source_text));
                    println!("{error:?}");
                    return;
                }
                println!("{parsed:#?}");
                println!();
            }
            AstKind::NewExpression(new_expr) => {
                if new_expr
                    .callee
                    .get_identifier_reference()
                    .filter(|ident| ident.name == "RegExp")
                    .is_some()
                {
                    println!("👻 TODO: new RegExp(...)");
                    println!();
                }
            }
            _ => {}
        }
    }
}

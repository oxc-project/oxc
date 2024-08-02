#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_module_lexer::ModuleLexer;
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// * create a `test.js`
// * `just example module_lexer

fn main() -> Result<(), String> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    println!("source:");
    println!("{source_text}");

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    let ModuleLexer { imports, exports, facade, has_module_syntax } =
        ModuleLexer::new().build(&ret.program);

    println!("\nimports:");
    for import in imports {
        println!("{import:?}");
    }

    println!("\nexports:");
    for export in exports {
        println!("{export:?}");
    }

    println!("\nfacade: {facade}");
    println!("has_module_syntax {has_module_syntax}");

    Ok(())
}

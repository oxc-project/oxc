use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example visitor`
// or `cargo watch -x "run -p oxc_parser --example visitor"`

fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
    }

    let program = ret.program;

    let mut ast_pass = ASTPass::default();
    ast_pass.visit_program(&program);
    println!("{ast_pass:?}");

    Ok(())
}

#[derive(Debug, Default)]
struct ASTPass {
    number_of_functions: usize,
    number_of_classes: usize,
}

impl<'a> Visit<'a> for ASTPass {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(_) => {
                self.number_of_functions += 1;
            }
            AstKind::Class(_) => {
                self.number_of_classes += 1;
            }
            _ => {}
        }
    }
}

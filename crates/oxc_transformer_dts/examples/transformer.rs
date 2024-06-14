use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer_dts::TransformerDts;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_transformer_dts --example transformer`
// or `just watch "run -p oxc_transformer_dts --example transformer"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.tsx".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    println!("Original:\n");
    println!("{source_text}\n");

    let program = ret.program;
    match TransformerDts::new(&allocator, path, &source_text, ret.trivias).build(&program) {
        Ok(dts) => {
            println!("Transformed dts:\n");
            println!("{dts}\n");
        }
        Err(errors) => {
            println!("Transformed dts failed:\n");
            for error in errors {
                println!("{error:?}");
            }
        }
    }
}

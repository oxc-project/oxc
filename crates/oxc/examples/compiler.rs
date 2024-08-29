#![allow(clippy::print_stdout)]

use std::{env, io, path::Path};

use oxc::{span::SourceType, Compiler};

// Instruction:
// 1. create a `test.js`
// 2. run either
//   * `cargo run -p oxc --example compiler --features="full"`
//   * `just watch 'run -p oxc --example compiler --features="full"'`

fn main() -> io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    match Compiler::default().execute(&source_text, source_type, path) {
        Ok(printed) => {
            println!("{printed}");
        }
        Err(errors) => {
            for error in errors {
                let error = error.with_source_code(source_text.to_string());
                println!("{error:?}");
            }
        }
    }

    Ok(())
}

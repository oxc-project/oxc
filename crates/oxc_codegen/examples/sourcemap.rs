use std::{env, path::Path};

use base64::{prelude::BASE64_STANDARD, Engine};
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// 1. create a `test.js`
// 2. run `cargo run -p oxc_codegen --example sourcemap`

fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return Ok(());
    }

    let codegen_options = CodegenOptions { enable_source_map: true, enable_typescript: true };

    let CodegenReturn { source_text, source_map } =
        Codegen::<false>::new(path.to_string_lossy().as_ref(), &source_text, codegen_options)
            .build(&ret.program);

    if let Some(source_map) = source_map {
        let result = source_map.to_json_string();
        let hash = BASE64_STANDARD.encode(format!(
            "{}\0{}{}\0{}",
            source_text.len(),
            source_text,
            result.len(),
            result
        ));
        println!("https://evanw.github.io/source-map-visualization/#{hash}");
    }

    Ok(())
}

#![allow(clippy::print_stdout)]
use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `just watch "cargo run -p oxc_parser --example parser"`

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    let show_ast = args.contains("--ast");
    let show_estree = args.contains("--estree");
    let show_comments = args.contains("--comments");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions { parse_regular_expression: true, ..ParseOptions::default() })
        .parse();
    let mut program = ret.program;

    if show_comments {
        println!("Comments:");
        for comment in &program.comments {
            let s = comment.content_span().source_text(&source_text);
            println!("{s}");
        }
    }

    if show_ast || show_estree {
        println!("AST:");
        if show_estree {
            Utf8ToUtf16::new().convert(&mut program);
        }
        println!("{}", serde_json::to_string_pretty(&program).unwrap());
    }

    if ret.errors.is_empty() {
        println!("Parsed Successfully.");
    } else {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
            println!("Parsed with Errors.");
        }
    }

    Ok(())
}

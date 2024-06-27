#![allow(clippy::print_stdout)]

use oxc_allocator::Allocator;
use oxc_regexp_parser::{ast, Parser, ParserOptions};

fn main() {
    let allocator = Allocator::default();

    for (pat, options) in [
        ("/ab/", ParserOptions::default()),
        ("/abc/i", ParserOptions::default()),
        ("/abcd/igv", ParserOptions::default()),
        ("/emo👈🏻ji/u", ParserOptions::default()),
        ("//", ParserOptions::default()),
        ("/duplicated-flags/ii", ParserOptions::default()),
        ("/invalid-flags/x", ParserOptions::default()),
    ] {
        println!("Test: {pat} + {options:?}");
        let parser = Parser::new(&allocator, pat, options);
        let ret = parser.parse();

        match ret {
            Ok(ast::RegExpLiteral { pattern, flags, .. }) => {
                println!("✨ {}", pattern.span.source_text(pat));
                println!("{pattern:#?}");
                println!("✨ {}", flags.span.source_text(pat));
                println!("{flags:?}");
            }
            Err(err) => println!("💥 {}", err.message),
        }
        println!();
    }
}

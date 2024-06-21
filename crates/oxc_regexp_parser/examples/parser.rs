use oxc_allocator::Allocator;
use oxc_regexp_parser::{ast, Parser, ParserOptions};

fn main() {
    let allocator = Allocator::default();

    for (pat, options) in [
        ("/abc/", ParserOptions::default()),
        ("/abc/iv", ParserOptions::default()),
        ("/duplicated-flags/ii", ParserOptions::default()),
        ("/invalid-flags/x", ParserOptions::default()),
    ] {
        println!("Test: {pat} + {options:?}");
        let parser = Parser::new(&allocator, pat, options);
        let ret = parser.parse();

        match ret {
            Ok(ast::RegExpLiteral { pattern, flags, .. }) => {
                println!("✨ {pattern:#?}");
                println!("✨ {flags:?}");
            }
            Err(err) => println!("💥 {}", err.message),
        }
        println!();
    }
}

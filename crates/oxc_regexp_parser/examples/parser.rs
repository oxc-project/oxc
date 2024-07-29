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
        ("/ab|c/i", ParserOptions::default()),
        ("/a|b+|c/i", ParserOptions::default()),
        ("/a{0}|b{1,2}|c{3,}/i", ParserOptions::default()),
        ("/(?=a)|(?<=b)|(?!c)|(?<!d)/i", ParserOptions::default()),
        (r"/\n\cM\0\x41\./", ParserOptions::default()),
        (r"/\n\cM\0\x41\u1234\./u", ParserOptions::default()),
        (r"/\n\cM\0\x41\u{1f600}\./u", ParserOptions::default()),
        (r"/a\k<f>x\1c/u", ParserOptions::default()),
        (r"/(cg)(?<n>cg)(?:g)/", ParserOptions::default()),
        (r"/{3}/", ParserOptions::default()), // Error
        (r"/Em🥹j/", ParserOptions::default()),
        (r"/^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$/", ParserOptions::default()),
        (r"/^(?<!ab)$/", ParserOptions::default()),
        (r"/a)/", ParserOptions::default()), // Error
        (r"/c]/", ParserOptions::default()),
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

#![allow(clippy::print_stdout)]

use oxc_allocator::Allocator;
use oxc_regexp_parser::{ast, Parser, ParserOptions};

fn main() {
    let allocator = Allocator::default();

    for source_text in [
        "/ab/",
        "/abc/i",
        "/abcd/igv",
        "/emo👈🏻ji/u",
        "/ab|c/i",
        "/a|b+|c/i",
        "/a{0}|b{1,2}|c{3,}/i",
        "/(?=a)|(?<=b)|(?!c)|(?<!d)/i",
        r"/\n\cM\0\x41\./",
        r"/\n\cM\0\x41\u1234\./u",
        r"/\n\cM\0\x41\u{1f600}\./u",
        r"/a\k<f>x\1c/u",
        r"/(cg)(?<n>cg)(?:g)/",
        r"/{3}/", // Error
        r"/Em🥹j/",
        r"/^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$/",
        r"/^(?<!ab)$/",
        r"/a)/", // Error
        r"/c]/",
        r"/[abc]/",
        r"/[|\]]/",
        r"/[a&&b]/v",
        r"/[a--b]/v",
        r"/[a&&&]/v", // Error
        r"/[a---]/v", // Error
        r"/[^a--b--c]/v",
        r"/[a[b[c[d[e[f[g[h[i[j[k[l]]]]]]]]]]]]/v",
        r"/[\q{abc|d|e|}]/v",
        r"/\p{Basic_Emoji}/v",
        r"/[[^\q{}]]/v",   // Error
        r"/(?<a>)(?<a>)/", // Error
    ] {
        println!("Parse: {source_text}");
        let parser = Parser::new(&allocator, source_text, ParserOptions::default());
        let ret = parser.parse();

        match ret {
            Ok(ast::RegExpLiteral { pattern, flags, .. }) => {
                println!("✨ {}", pattern.span.source_text(source_text));
                println!("{pattern:#?}");
                println!("✨ {}", flags.span.source_text(source_text));
                println!("{flags:?}");
            }
            Err(err) => println!("💥 {err}"),
        }
        println!();
    }
}

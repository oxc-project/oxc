use oxc_allocator::Allocator;
use oxc_regexp_parser::{ast, Parser};

fn main() {
    println!("🍀 Hello parser!");

    let allocator = Allocator::default();

    let parser = Parser::new(&allocator, "/abc/i");
    let ret = parser.parse();

    match ret {
        Ok(ast::RegExpLiteral { pattern, flags, .. }) => {
            println!("✨ PAT: {pattern:#?}");
            println!("✨ FLG: {flags:#?}");
        }
        Err(err) => println!("💥 {err:#?}"),
    }
}

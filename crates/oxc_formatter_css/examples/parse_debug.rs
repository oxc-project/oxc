//! Dump oxc-css-parser's AST for a CSS/SCSS/Less snippet.
//!
//! ```sh
//! cargo run -p oxc_formatter_css --example parse_debug -- file.css
//! echo 'a { color: red }' | cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss -
//! ```
#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::io::Read;

use oxc_allocator::Allocator;
use oxc_css_parser::{ParserBuilder, Syntax, ast::Stylesheet};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let syntax =
        match args.opt_value_from_str::<_, String>("--syntax").unwrap().as_deref().unwrap_or("css")
        {
            "scss" => Syntax::Scss,
            "less" => Syntax::Less,
            _ => Syntax::Css,
        };
    let name: String = args.free_from_str().unwrap_or_else(|_| "-".to_string());

    let source = if name == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap();
        buf
    } else {
        std::fs::read_to_string(&name).unwrap()
    };

    let allocator = Allocator::default();
    let mut parser = ParserBuilder::new(&allocator, &source).syntax(syntax).comments().build();
    let result = parser.parse::<Stylesheet>();
    let errors = parser.recoverable_errors().to_vec();
    let comments = parser.comments().to_vec();
    drop(parser);
    match result {
        Ok(stylesheet) => {
            println!("{stylesheet:#?}");
            println!("--- comments ---");
            println!("{comments:#?}");
            println!("--- recoverable errors ---");
            println!("{errors:#?}");
        }
        Err(err) => eprintln!("Parse error: {err:?}"),
    }
}

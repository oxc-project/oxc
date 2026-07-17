//! This example outlines using `oxc-graphql-parser` with [annotate-snippets], the error
//! printing library used by the Rust compiler.
//!
//! This allows for a lot of control over how you would like your error output
//! to look before your print them all out.
//!
//! [annotate-snippets]: https://docs.rs/annotate-snippets/0.12.0/annotate_snippets/

use annotate_snippets::AnnotationKind;
use annotate_snippets::Level;
use annotate_snippets::Renderer;
use annotate_snippets::Snippet;
use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Parser;
use std::fs;
use std::path::Path;

fn parse_schema() {
    let file = Path::new("crates/oxc_graphql_parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    // this is a nice to have for errors for displaying error origin.
    let file_name = file
        .file_name()
        .expect("Could not get file name.")
        .to_str()
        .expect("Could not get &str from file name.");
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &src);
    let ast = parser.parse();

    // each err comes with the two pieces of data you need for diagnostics:
    // - message (err.message())
    // - index (err.index())
    for err in ast.errors() {
        let snippet = Level::ERROR.primary_title(err.message()).element(
            Snippet::source(&src).line_start(0).path(file_name).fold(true).annotation(
                AnnotationKind::Primary
                    .span(err.index()..err.index() + err.data().len())
                    .label(err.message()),
            ),
        );

        let renderer = Renderer::styled();
        println!("{}\n\n", renderer.render(&[snippet]));
    }
}

fn main() {
    parse_schema();
}

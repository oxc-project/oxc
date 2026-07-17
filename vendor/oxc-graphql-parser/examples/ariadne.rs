//! This example describes how to use `oxc-graphql-parser` with
//! [`ariadne`](https://docs.rs/ariadne/0.3.0/ariadne) diagnostic library.

use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Parser;
use std::fs;
use std::path::Path;

fn parse_schema() {
    let file = Path::new("crates/oxc_graphql_parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    // This is really useful for display the src path within the diagnostic.
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
        // We need to create a report and print that individually, as the error
        // slice can have many errors.
        let start = err.index();
        let end = start + err.data().len();
        Report::build(ReportKind::Error, (file_name, start..end))
            .with_message(err.message())
            .with_label(Label::new((file_name, start..end)).with_message(err.message()))
            .finish()
            .eprint((file_name, Source::from(&src)))
            .unwrap();
    }
}

fn main() {
    parse_schema();
}

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn transform(path: &Path, source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let program = Parser::new(&allocator, source_text, source_type).parse().program;
    CodeGenerator::new().build(&program).source_text
}

#[test]
fn snapshots() {
    insta::glob!("fixtures/*.{ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();
        let snapshot = transform(path, &source_text);
        let name = path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}

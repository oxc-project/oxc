use std::{fs, path::Path};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::project_root;
use oxc_transformer::{TransformOptions, TransformTarget, Transformer};

/// # Panics
pub fn babel(name: &str) {
    let root = project_root().join("tasks/coverage/babel/packages");
    let root = root.join(name).join("test/fixtures");
    let paths = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "input.js")
        .map(|e| e.path().parent().unwrap().to_path_buf())
        .collect::<Vec<_>>();
    for path in paths {
        babel_test(&path);
    }
}

fn babel_test(path: &Path) {
    let input_path = path.join("input.js");
    let output_path = path.join("output.js");
    let source_text = fs::read_to_string(&input_path).unwrap();
    let expected = fs::read_to_string(output_path).unwrap();

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&input_path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let transform_options = TransformOptions { target: TransformTarget::ES2015 };
    let program = allocator.alloc(ret.program);
    Transformer::new(&allocator, &transform_options).build(program);

    let formatter_options = FormatterOptions::default();
    let printed = Formatter::new(source_text.len(), formatter_options).build(program);

    let printed = printed.replace(|c: char| c.is_ascii_whitespace(), "");
    let expected = expected.replace(|c: char| c.is_ascii_whitespace(), "");
    assert_eq!(printed, expected, "{path:?}");
}

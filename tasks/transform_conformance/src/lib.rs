use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::{normalize_path, project_root};
use oxc_transformer::{TransformOptions, TransformTarget, Transformer};

/// # Panics
pub fn babel() {
    let root = project_root().join("tasks/coverage/babel/packages");

    let cases = [
        // ES2019
        "babel-plugin-transform-optional-catch-binding",
        // ES2016
        "babel-plugin-transform-exponentiation-operator",
    ];

    // Get all fixtures
    let mut paths = cases
        .into_iter()
        .flat_map(|case| {
            let root = root.join(case).join("test/fixtures");
            WalkDir::new(root).into_iter()
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "input.js")
        .map(|e| e.path().parent().unwrap().to_path_buf())
        .collect::<Vec<_>>();
    paths.sort_unstable();

    // Run the test
    let (passed, failed): (Vec<PathBuf>, Vec<PathBuf>) =
        paths.into_iter().partition(|path| babel_test(path));

    // Snapshot
    let mut snapshot = String::new();
    for path in failed {
        snapshot.push_str("Failed: ");
        snapshot.push_str(&normalize_path(path.strip_prefix(&root).unwrap()));
        snapshot.push('\n');
    }
    snapshot.push('\n');
    for path in passed {
        snapshot.push_str("Passed: ");
        snapshot.push_str(&normalize_path(path.strip_prefix(&root).unwrap()));
        snapshot.push('\n');
    }
    let path = project_root().join("tasks/transform_conformance/babel.snap");
    let mut file = File::create(path).unwrap();
    file.write_all(snapshot.as_bytes()).unwrap();
}

fn babel_test(path: &Path) -> bool {
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
    let transformed = Formatter::new(source_text.len(), formatter_options).build(program);

    remove_whitespace(&transformed) == remove_whitespace(&expected)
    // if !passed {
    // println!("{input_path:?}");
    // println!("Transformed:\n");
    // println!("{transformed}");
    // println!("Expected:\n");
    // println!("{expected}");
    // }
}

fn remove_whitespace(s: &str) -> String {
    s.replace(|c: char| c.is_ascii_whitespace(), "")
}

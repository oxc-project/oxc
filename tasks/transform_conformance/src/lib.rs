use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_parser::Parser;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, project_root};
use oxc_transformer::{TransformOptions, TransformTarget, Transformer};

/// # Panics
pub fn babel() {
    let root = project_root().join("tasks/coverage/babel/packages");

    let cases = [
        // ES2024
        "babel-plugin-transform-unicode-sets-regex",
        // ES2021
        "babel-plugin-transform-numeric-separator",
        // ES2020
        "babel-plugin-transform-export-namespace-from",
        // ES2019
        "babel-plugin-transform-optional-catch-binding",
        // ES2016
        "babel-plugin-transform-exponentiation-operator",
    ];

    let mut snapshot = String::new();

    // Get all fixtures
    for case in cases {
        let root = root.join(case).join("test/fixtures");
        let mut paths = WalkDir::new(&root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.path().file_stem().is_some_and(|name| name == "input")
                    && e.path()
                        .extension()
                        .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
            })
            .map(walkdir::DirEntry::into_path)
            .collect::<Vec<_>>();
        paths.sort_unstable();

        // Run the test
        let (passed, failed): (Vec<PathBuf>, Vec<PathBuf>) =
            paths.into_iter().partition(|path| babel_test(path));

        // Snapshot
        snapshot.push_str("# ");
        snapshot.push_str(case);
        snapshot.push('\n');
        if failed.is_empty() {
            snapshot.push_str("[All passed]\n");
        }
        for path in failed {
            snapshot.push_str("* Failed: ");
            snapshot.push_str(&normalize_path(path.strip_prefix(&root).unwrap()));
            snapshot.push('\n');
        }
        for path in passed {
            snapshot.push_str("* Passed: ");
            snapshot.push_str(&normalize_path(path.strip_prefix(&root).unwrap()));
            snapshot.push('\n');
        }
        snapshot.push('\n');
    }

    let path = project_root().join("tasks/transform_conformance/babel.snap.md");
    let mut file = File::create(path).unwrap();
    file.write_all(snapshot.as_bytes()).unwrap();
}

fn babel_test(input_path: &Path) -> bool {
    let extension = input_path.extension().unwrap().to_str().unwrap();
    let output_path = input_path.parent().unwrap().join(format!("output.{extension}"));
    let source_text = fs::read_to_string(input_path).unwrap();

    let expected = fs::read_to_string(output_path).ok();

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(input_path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if let Some(expected) = &expected {
        let transform_options = TransformOptions { target: TransformTarget::ES2015 };
        let program = allocator.alloc(ret.program);
        Transformer::new(&allocator, &transform_options).build(program);

        let formatter_options = FormatterOptions::default();
        let transformed = Formatter::new(source_text.len(), formatter_options).build(program);
        // if !passed {
        // println!("{input_path:?}");
        // println!("Transformed:\n");
        // println!("{transformed}");
        // println!("Expected:\n");
        // println!("{expected}");
        // }
        return remove_whitespace(&transformed) == remove_whitespace(expected);
    }

    ret.errors.is_empty()
}

fn remove_whitespace(s: &str) -> String {
    s.replace(|c: char| c.is_ascii_whitespace(), "")
}

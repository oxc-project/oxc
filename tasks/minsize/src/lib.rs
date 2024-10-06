#![allow(clippy::print_stdout, clippy::print_stderr)]
use std::{
    fs::File,
    io::{self, Write},
};

use flate2::{write::GzEncoder, Compression};
use humansize::{format_size, DECIMAL};
use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::{project_root, TestFile, TestFiles};
use rustc_hash::FxHashMap;

// #[test]
// #[cfg(any(coverage, coverage_nightly))]
// fn test() {
// run().unwrap();
// }

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::minifier();

    let path = project_root().join("tasks/minsize/minsize.snap");

    // Data copied from https://github.com/privatenumber/minification-benchmarks
    let targets = FxHashMap::<&str, &str>::from_iter([
        ("react.development.js", "23.70 kB"),
        ("moment.js", "59.82 kB"),
        ("jquery.js", "90.07 kB"),
        ("vue.js", "118.14 kB"),
        ("lodash.js", "72.48 kB"),
        ("d3.js", "270.13 kB"),
        ("bundle.min.js", "458.89 kB"),
        ("three.js", "646.76 kB"),
        ("victory.js", "724.14 kB"),
        ("echarts.js", "1.01 MB"),
        ("antd.js", "2.31 MB"),
        ("typescript.js", "3.49 MB"),
    ]);

    let gzip_targets = FxHashMap::<&str, &str>::from_iter([
        ("react.development.js", "8.54 kB"),
        ("moment.js", "19.33 kB"),
        ("jquery.js", "31.95 kB"),
        ("vue.js", "44.37 kB"),
        ("lodash.js", "26.20 kB"),
        ("d3.js", "90.80 kB"),
        ("bundle.min.js", "126.71 kB"),
        ("three.js", "163.73 kB"),
        ("victory.js", "181.07 kB"),
        ("echarts.js", "331.56 kB"),
        ("antd.js", "488.28 kB"),
        ("typescript.js", "915.50 kB"),
    ]);

    let mut out = String::new();
    out.push_str(&format!(
        "{:width$} | {:width$} | {:width$} | {:width$} | {:width$}\n",
        "Original",
        "Minified",
        "esbuild",
        "Gzip",
        "esbuild",
        width = 10
    ));
    out.push('\n');

    for file in files.files() {
        let minified = minify_twice(file);
        let s = format!(
            "{:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$}\n\n",
            format_size(file.source_text.len(), DECIMAL),
            format_size(minified.len(), DECIMAL),
            targets[file.file_name.as_str()],
            format_size(gzip_size(&minified), DECIMAL),
            gzip_targets[file.file_name.as_str()],
            &file.file_name,
            width = 10
        );
        out.push_str(&s);
    }

    println!("{out}");

    let mut snapshot = File::create(path)?;
    snapshot.write_all(out.as_bytes())?;
    snapshot.flush()?;
    Ok(())
}

fn minify_twice(file: &TestFile) -> String {
    let source_type = SourceType::from_path(&file.file_name).unwrap();
    let options = MinifierOptions {
        mangle: true,
        compress: CompressOptions { evaluate: false, ..CompressOptions::default() },
    };
    // let source_text1 = minify(&file.source_text, source_type, options);
    // let source_text2 = minify(&source_text1, source_type, options);
    // assert!(source_text1 == source_text2, "Minification failed for {}", &file.file_name);
    // source_text2
    minify(&file.source_text, source_type, options)
}

fn minify(source_text: &str, source_type: SourceType, options: MinifierOptions) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let ret = Minifier::new(options).build(&allocator, program);
    CodeGenerator::new()
        .with_options(CodegenOptions { minify: true, ..CodegenOptions::default() })
        .with_mangler(ret.mangler)
        .build(program)
        .code
}

fn gzip_size(s: &str) -> usize {
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(s.as_bytes()).unwrap();
    let s = e.finish().unwrap();
    s.len()
}

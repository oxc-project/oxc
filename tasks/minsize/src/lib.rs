#![expect(clippy::print_stdout)]

use std::{
    fmt::Write as _,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use cow_utils::CowUtils;
use flate2::{Compression, write::GzEncoder};
use humansize::{DECIMAL, format_size};
use pico_args::Arguments;
use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::{TestFile, TestFiles, project_root};
use oxc_transformer_plugins::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

#[derive(Debug, Clone, Copy)]
struct Options {
    compress_only: bool,
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let mut args = Arguments::from_env();

    let options = Options { compress_only: args.contains("--compress-only") };

    let marker = args.free_from_str().unwrap_or_else(|_| "default".to_string());

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

    let width = 10;
    writeln!(
        out,
        "{:width$} | {:width$} | {:width$} | {:width$} | {:width$} |",
        "",
        "Oxc",
        "ESBuild",
        "Oxc",
        "ESBuild",
        width = width,
    )
    .unwrap();
    writeln!(
        out,
        "{:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$}",
        "Original",
        "minified",
        "minified",
        "gzip",
        "gzip",
        "Iterations",
        "File",
        width = width,
    )
    .unwrap();

    let fixture_width = files
        .files()
        .iter()
        .max_by(|x, y| x.file_name.len().cmp(&y.file_name.len()))
        .unwrap()
        .file_name
        .len();
    out.push_str(&str::repeat("-", width * 5 + fixture_width + 15));
    out.push('\n');

    let save_path = Path::new("./target/minifier").join(marker);

    for file in files.files() {
        let (minified, iterations) = minify_twice(file, options);

        fs::create_dir_all(&save_path).unwrap();
        fs::write(save_path.join(&file.file_name), &minified).unwrap();

        let s = format!(
            "{:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} \n\n",
            format_size(file.source_text.len(), DECIMAL),
            format_size(minified.len(), DECIMAL),
            targets[file.file_name.as_str()],
            format_size(gzip_size(&minified), DECIMAL),
            gzip_targets[file.file_name.as_str()],
            iterations,
            &file.file_name,
            width = width
        );
        out.push_str(&s);
    }

    println!("{out}");

    if !options.compress_only {
        let mut snapshot = File::create(path)?;
        snapshot.write_all(out.as_bytes())?;
        snapshot.flush()?;
    }
    Ok(())
}

fn minify_twice(file: &TestFile, options: Options) -> (String, u8) {
    let source_type = SourceType::cjs();
    let (code1, iterations) = minify(&file.source_text, source_type, options);
    let (code2, _) = minify(&code1, source_type, options);
    assert_eq_minified_code(&code1, &code2, &file.file_name);
    (code2, iterations)
}

fn minify(source_text: &str, source_type: SourceType, options: Options) -> (String, u8) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let _ = ReplaceGlobalDefines::new(
        &allocator,
        ReplaceGlobalDefinesConfig::new(&[("process.env.NODE_ENV", "'development'")]).unwrap(),
    )
    .build(scoping, &mut program);
    let ret = Minifier::new(MinifierOptions {
        mangle: (!options.compress_only).then(MangleOptions::default),
        compress: Some(CompressOptions::default()),
    })
    .build(&allocator, &mut program);
    let code = Codegen::new()
        .with_options(CodegenOptions { minify: !options.compress_only, ..CodegenOptions::minify() })
        .with_scoping(ret.scoping)
        .build(&program)
        .code;
    (code, ret.iterations)
}

fn gzip_size(s: &str) -> usize {
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(s.as_bytes()).unwrap();
    let s = e.finish().unwrap();
    s.len()
}

fn assert_eq_minified_code(s1: &str, s2: &str, filename: &str) {
    if s1 != s2 {
        let left = normalize_minified_code(s1);
        let right = normalize_minified_code(s2);
        similar_asserts::assert_eq!(left, right, "Minification failed for {filename}");
    }
}

fn normalize_minified_code(code: &str) -> String {
    code.cow_replace(";", ";\n").cow_replace(",", ",\n").into_owned()
}

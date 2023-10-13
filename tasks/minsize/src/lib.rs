#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    fs::File,
    io::{self, Write},
};

use brotlic::{BlockSize, BrotliEncoderOptions, CompressorWriter, Quality, WindowSize};
use flate2::{write::GzEncoder, Compression};
use humansize::{format_size, DECIMAL};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::{project_root, TestFile, TestFiles};

// #[test]
// #[cfg(any(coverage, coverage_nightly))]
// fn test() {
// run().unwrap();
// }

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::new();

    let path = project_root().join("tasks/minsize/minsize.snap");

    let mut out = String::new();
    out.push_str(&format!(
        "{:width$} -> {:width$} -> {:width$}",
        "Original",
        "Minified",
        "Gzip",
        width = 10
    ));
    out.push_str(&format!(" {:width$}\n", "Brotli", width = 10));
    for file in files.files() {
        let minified = minify_twice(file);
        let s = format!(
            "{:width$} -> {:width$} -> {:width$} {:width$} {}\n\n",
            format_size(file.source_text.len(), DECIMAL),
            format_size(minified.len(), DECIMAL),
            format_size(gzip_size(&minified), DECIMAL),
            format_size(brotli_size(&minified), DECIMAL),
            &file.file_name,
            width = 10
        );
        out.push_str(&s);
    }

    let mut snapshot = File::create(path)?;
    snapshot.write_all(out.as_bytes())?;
    snapshot.flush()?;
    Ok(())
}

fn minify_twice(file: &TestFile) -> String {
    let source_type = SourceType::from_path(&file.file_name).unwrap();
    let options = MinifierOptions {
        compress: CompressOptions { evaluate: false, ..CompressOptions::default() },
        ..MinifierOptions::default()
    };
    let source_text1 = minify(&file.source_text, source_type, options);
    let source_text2 = minify(&source_text1, source_type, options);
    assert!(source_text1 == source_text2, "Minification failed for {}", &file.file_name);
    source_text2
}

fn minify(source_text: &str, source_type: SourceType, options: MinifierOptions) -> String {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, source_text, source_type).parse().program;
    let program = allocator.alloc(program);
    Minifier::new(options).build(&allocator, program);
    Codegen::<true>::new(source_text.len(), CodegenOptions).build(program)
}

fn gzip_size(s: &str) -> usize {
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(s.as_bytes()).unwrap();
    let s = e.finish().unwrap();
    s.len()
}

fn brotli_size(s: &str) -> usize {
    let encoder = BrotliEncoderOptions::new()
        .quality(Quality::best())
        .window_size(WindowSize::best())
        .block_size(BlockSize::best())
        .build()
        .unwrap();

    let mut e = CompressorWriter::with_encoder(encoder, Vec::new());
    e.write_all(s.as_bytes()).unwrap();
    let s = e.into_inner().unwrap();
    s.len()
}

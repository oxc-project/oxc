use std::{
    fs::File,
    io::{self, Write},
};

use flate2::{write::GzEncoder, Compression};
use humansize::{format_size, DECIMAL};
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::project_root;
use oxc_tasks_common::{TestFile, TestFiles};

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::new();

    let path = project_root().join("tasks/minsize/minsize.snap");

    let mut out = String::new();
    out.push_str(&format!("{:width$} -> {:width$} -> Gzip\n", "Original", "Minified", width = 10));
    for file in files.files() {
        let minified = minify(file);
        let s = format!(
            "{:width$} -> {:width$} -> {:width$} {}\n",
            format_size(file.source_text.len(), DECIMAL),
            format_size(minified.len(), DECIMAL),
            format_size(gzip_size(&minified), DECIMAL),
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

fn minify(file: &TestFile) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&file.file_name).unwrap();
    let source_text = &file.source_text;
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let semantic_ret = SemanticBuilder::new(source_text, source_type, &ret.trivias).build(program);
    let printer_options = PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() };
    Printer::new(source_text.len(), printer_options)
        .with_symbol_table(&semantic_ret.semantic.symbols(), true)
        .build(program)
}

fn gzip_size(s: &str) -> usize {
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(s.as_bytes()).unwrap();
    let s = e.finish().unwrap();
    s.len()
}

use std::{
    fs::File,
    io::{self, Write},
};

use humansize::{format_size, DECIMAL};
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::project_root;
use oxc_tasks_common::{TestFile, TestFiles};

#[test]
fn test() {
    run().unwrap();
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::new();

    let path = project_root().join("tasks/minsize/minsize.snap");

    let mut out = String::new();
    for file in files.files() {
        let size = get_size(file);
        let s = format!(
            "{:width$} -> {:width$} - {}\n",
            format_size(file.source_text.len(), DECIMAL),
            format_size(size, DECIMAL),
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

fn get_size(file: &TestFile) -> usize {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&file.file_name).unwrap();
    let source_text = &file.source_text;
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let _semantic = SemanticBuilder::new(source_text, source_type, &ret.trivias);
    let printer_options = PrinterOptions::default();
    let printed = Printer::new(source_text.len(), printer_options).build(program);
    printed.len()
}

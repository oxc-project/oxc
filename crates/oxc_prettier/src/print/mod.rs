mod command;
mod printer;

use oxc_allocator::Allocator;

use crate::{PrettierOptions, ir::Doc};

pub fn print_doc_to_string<'a>(
    allocator: &'a Allocator,
    doc: Doc<'a>,
    options: PrettierOptions,
    out_size_hint: usize,
) -> String {
    printer::Printer::new(allocator, doc, options, out_size_hint).build()
}

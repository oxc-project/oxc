//! Format a CSS/SCSS/Less fragment through the embedded entry point
//! (`format_to_ir`), the dispatcher path oxfmt uses for css-in-js.
//! Unlike `css_formatter`, this tolerates `` `PLACEHOLDER-N` ``
//! markers in value/selector position.
//!
//! ```sh
//! cargo run -p oxc_formatter_css --example embedded_debug -- [filename]
//! ```
#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{Document, EmbeddedContext, FormatOptions, Printer, UniqueGroupIdBuilder};
use oxc_formatter_css::{CssFormatOptions, CssVariant, format_to_ir};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.scss".to_string());
    let path = Path::new(&name);

    let source_text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()));

    let variant = match path.extension().and_then(|e| e.to_str()) {
        Some("less") => CssVariant::Less,
        // The css-in-js dispatcher always parses as SCSS, mirror it.
        _ => CssVariant::Scss,
    };
    // Match Prettier's default print width for side-by-side comparison.
    let line_width = oxc_formatter_core::LineWidth::try_from(80).unwrap();
    let options = CssFormatOptions { variant, line_width, ..CssFormatOptions::default() };

    let allocator = Allocator::new();
    let group_id_builder = UniqueGroupIdBuilder::default();
    let ctx = EmbeddedContext {
        allocator: &allocator,
        group_id_builder: &group_id_builder,
        dispatcher: None,
    };

    match format_to_ir(&ctx, &source_text, options) {
        Ok(embedded) => {
            let document = Document::new(embedded.ir, Vec::new());
            document.propagate_expand();
            if std::env::var("DUMP_IR").is_ok() {
                for el in document.elements() {
                    eprintln!("{el:?}");
                }
            }
            let (elements, tailwind_classes) = document.into_elements_and_tailwind_classes();
            match Printer::with_capacity(
                source_text.len(),
                options.as_print_options(),
                &tailwind_classes,
            )
            .print(elements)
            {
                Ok(printed) => println!("{}", printed.into_code()),
                Err(err) => eprintln!("Print error: {err:?}"),
            }
        }
        Err(diagnostic) => eprintln!("Parse error: {diagnostic:?}"),
    }
}

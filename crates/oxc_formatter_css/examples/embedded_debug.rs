//! Format an SCSS fragment through the embedded entry point
//! (`format_to_ir`), the dispatcher path oxfmt uses for css-in-js.
//! The dispatcher always parses as SCSS, so this example is hardcoded
//! to `CssVariant::Scss`. Unlike `css_formatter`, this tolerates
//! `` `PLACEHOLDER-N` `` markers in value/selector position.
//!
//! ```sh
//! cargo run -p oxc_formatter_css --example embedded_debug -- [filename]
//! ```
#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{Document, FormatOptions, FormatSession, InputKind};
use oxc_formatter_css::{CssFormatOptions, CssVariant, format_to_ir};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.scss".to_string());
    let path = Path::new(&name);

    let source_text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()));

    // Match Prettier's default print width for side-by-side comparison.
    let line_width = oxc_formatter_core::LineWidth::try_from(80).unwrap();
    // The css-in-js dispatcher always parses as SCSS.
    let options =
        CssFormatOptions { variant: CssVariant::Scss, line_width, ..CssFormatOptions::default() };

    let allocator = Allocator::new();
    let session = FormatSession::new(&allocator, InputKind::Fragment);

    match format_to_ir(&session, &source_text, options, /* template_placeholders */ true) {
        Ok(embedded) => {
            let document = Document::new(embedded.ir, Vec::new());
            // `elements` borrows the arena (not the document) and group modes are `Cell`s,
            // so after `print` finalizes, the slice shows the finalized IR.
            let elements = std::env::var("DUMP_IR").is_ok().then(|| document.elements());
            match document.print(source_text.len(), options.as_print_options()) {
                Ok(printed) => println!("{}", printed.into_code()),
                Err(err) => eprintln!("Print error: {err:?}"),
            }
            if let Some(elements) = elements {
                for el in elements {
                    eprintln!("{el:?}");
                }
            }
        }
        Err(diagnostic) => eprintln!("Parse error: {diagnostic:?}"),
    }
}

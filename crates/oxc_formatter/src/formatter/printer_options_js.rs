use oxc_formatter_core::PrinterOptions;

use crate::JsFormatOptions;

impl<'a> From<&'a JsFormatOptions> for PrinterOptions {
    fn from(options: &'a JsFormatOptions) -> Self {
        PrinterOptions::default()
            .with_indent_style(options.indent_style)
            .with_indent_width(options.indent_width)
            .with_print_width(options.line_width.into())
            .with_line_ending(options.line_ending)
    }
}

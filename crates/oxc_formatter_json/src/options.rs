use oxc_formatter_core::formatter::printer::{AsPrinterOptions, PrinterOptions};
use oxc_formatter_core::{IndentStyle, IndentWidth, IndentWidthProvider, LineEnding, LineWidth};

#[derive(Debug, Clone, Copy)]
pub struct JsonFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_ending: LineEnding,
    pub line_width: LineWidth,
    pub always_expand: bool,
}

impl Default for JsonFormatOptions {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Space,
            indent_width: IndentWidth::default(),
            line_ending: LineEnding::Lf,
            line_width: LineWidth::default(),
            always_expand: false,
        }
    }
}

impl AsPrinterOptions for JsonFormatOptions {
    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent_style(self.indent_style)
            .with_indent_width(self.indent_width)
            .with_print_width(self.line_width.into())
            .with_line_ending(self.line_ending)
    }
}

impl IndentWidthProvider for JsonFormatOptions {
    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }
}

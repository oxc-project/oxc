use crate::{
    FormatContext, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// Simple format context useful for testing.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct SimpleFormatContext<'src> {
    options: SimpleFormatOptions,
    source_code: &'src str,
    tailwind_classes: Vec<String>,
}

impl<'src> SimpleFormatContext<'src> {
    pub fn new(options: SimpleFormatOptions) -> Self {
        Self { options, source_code: "", tailwind_classes: Vec::new() }
    }

    #[must_use]
    pub fn with_source_code(mut self, code: &'src str) -> Self {
        self.source_code = code;
        self
    }

    /// Set the collected sorted Tailwind CSS classes used when rendering
    /// `FormatElement::TailwindClass` entries.
    pub fn set_tailwind_classes(&mut self, classes: Vec<String>) {
        self.tailwind_classes = classes;
    }
}

impl FormatContext for SimpleFormatContext<'_> {
    type Options = SimpleFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        self.source_code
    }

    fn get_tailwind_class(&self, idx: usize) -> Option<&str> {
        self.tailwind_classes.get(idx).map(String::as_str)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct SimpleFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
}

impl FormatOptions for SimpleFormatOptions {
    fn indent_style(&self) -> IndentStyle {
        self.indent_style
    }

    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }

    fn line_width(&self) -> LineWidth {
        self.line_width
    }

    fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent_style(self.indent_style)
            .with_indent_width(self.indent_width)
            .with_line_ending(self.line_ending)
            .with_print_width(self.line_width.into())
    }
}

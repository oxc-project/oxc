use oxc_formatter_core::{
    FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// Format options for GraphQL.
///
/// Prettier's `graphql` language consumes only the shared layout options plus
/// `bracketSpacing` (see `prettier/src/language-graphql/options.js`).
/// Other Prettier options (`trailingComma`, `singleQuote`, ...) have no effect on GraphQL output.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct GraphqlFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    /// Spaces inside `{ ... }` of object values. Mirrors Prettier's `bracketSpacing`.
    pub bracket_spacing: BracketSpacing,
}

/// Whether to insert spaces around brackets in object values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BracketSpacing(bool);

impl BracketSpacing {
    pub fn value(self) -> bool {
        self.0
    }
}

impl Default for BracketSpacing {
    fn default() -> Self {
        Self(true)
    }
}

impl From<bool> for BracketSpacing {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl FormatOptions for GraphqlFormatOptions {
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

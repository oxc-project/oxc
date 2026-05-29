use oxc_formatter_core::{
    FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// JSON parser variant.
///
/// All variants share the same lenient input acceptance:
/// the underlying parser is the JS expression parser, so comments, trailing commas,
/// single quotes, and unquoted keys are all parseable regardless of variant.
/// What differs is the output formatting and a few variant-specific behaviors noted below.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum JsonVariant {
    /// Prettier's `parser: json` equivalent.
    /// Output: double-quoted strings, quoted object keys,
    /// trailing commas forced off (regardless of user option).
    #[default]
    Json,
    /// Prettier's `parser: jsonc` equivalent.
    /// Output: double-quoted strings, quoted object keys;
    /// trailing commas follow the user option.
    /// Empty input is allowed.
    Jsonc,
    /// Prettier's `parser: json5` equivalent.
    /// Output: object keys may stay unquoted,
    /// string quote style and trailing commas follow user options.
    Json5,
    /// Prettier's `parser: json-stringify` equivalent.
    /// Output: `JSON.parse()`-compatible.
    /// double-quoted strings, quoted keys, no trailing commas,
    /// always pretty-printed with hard line breaks between entries.
    /// The only variant that rejects comments at parse time.
    JsonStringify,
}

/// Whether objects keep their authored multi-line shape or collapse to one line when they fit.
/// Mirrors Prettier's `objectWrap` option for the `json` parser.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum Expand {
    /// `objectWrap: "preserve"`.
    /// An object stays multi-line if there's a newline after `{` in the source;
    /// otherwise it collapses when it fits.
    #[default]
    Auto,
    /// `objectWrap: "collapse"`.
    /// Objects collapse when they fit regardless of the authored shape.
    Never,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JsonFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    pub variant: JsonVariant,
    pub bracket_spacing: bool,
    pub expand: Expand,
}

impl Default for JsonFormatOptions {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            indent_width: IndentWidth::default(),
            line_width: LineWidth::default(),
            line_ending: LineEnding::default(),
            variant: JsonVariant::default(),
            bracket_spacing: true,
            expand: Expand::default(),
        }
    }
}

impl FormatOptions for JsonFormatOptions {
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

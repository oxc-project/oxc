use oxc_formatter_core::{
    BracketSpacing, Expand, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
    PrinterOptions, TrailingCommas,
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
    /// Output: double-quoted strings, quoted object keys,
    /// trailing commas follow the user option.
    /// Empty input is allowed.
    Jsonc,
    /// Prettier's `parser: json5` equivalent.
    /// Output: object keys may stay unquoted,
    /// string quote style and trailing commas follow user options.
    Json5,
    /// Prettier's `parser: json-stringify` equivalent.
    /// Output: `JSON.parse()`-compatible,
    /// double-quoted strings, quoted keys, no trailing commas,
    /// always pretty-printed with hard line breaks between entries.
    /// The only variant that rejects comments at parse time.
    JsonStringify,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct JsonFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    pub variant: JsonVariant,
    pub bracket_spacing: BracketSpacing,
    pub expand: Expand,
    pub trailing_commas: TrailingCommas,
}

impl JsonFormatOptions {
    /// Whether a trailing comma may follow the last entry of a multi-line object/array,
    /// per the active variant and [`Self::trailing_commas`].
    ///
    /// `json` and `json-stringify` never emit one, but Prettier achieves this differently:
    /// - `json`: the option is force-normalized to `trailingComma: "none"`.
    /// - `json-stringify`: a separate `estree-json` printer is used
    ///   - Always hard-breaks entries and never emits a trailing comma (the option is irrelevant)
    ///   - So full `json-stringify` parity will also need always-expand + no concise arrays, not just this flag
    ///
    /// `jsonc` and `json5` follow the user option (`all`/`es5` both emit, `none` does not).
    /// Both go through Prettier's shared `estree` printer with `shouldPrintTrailingComma` fixed at the `es5` level,
    /// which is why `all` and `es5` are indistinguishable here.
    pub fn allow_trailing_comma(&self) -> bool {
        match self.variant {
            JsonVariant::Json | JsonVariant::JsonStringify => false,
            JsonVariant::Jsonc | JsonVariant::Json5 => !self.trailing_commas.is_none(),
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

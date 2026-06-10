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
    // Used by: JSON, JSONC, JSON5
    pub bracket_spacing: BracketSpacing,
    // Used by: JSON, JSONC, JSON5
    pub expand: Expand,
    // Used by: JSONC, JSON5
    pub trailing_commas: TrailingCommas,
    // Used by: JSON5
    pub single_quote: SingleQuote,
    // Used by: JSON5
    pub quote_props: QuoteProps,
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
    /// `jsonc` and `json5` follow the user option (`Always` emits, `Never` does not).
    /// Both go through Prettier's shared `estree` printer with `shouldPrintTrailingComma` fixed at the `es5` level,
    /// which is why Prettier's `all` and `es5` are indistinguishable here (both map to `Always`).
    pub fn allow_trailing_comma(&self) -> bool {
        match self.variant {
            JsonVariant::Json | JsonVariant::JsonStringify => false,
            JsonVariant::Jsonc | JsonVariant::Json5 => {
                matches!(self.trailing_commas, TrailingCommas::Always)
            }
        }
    }

    /// The quote byte (`b'"'` / `b'\''`) to enclose a string literal whose body is `inner`
    /// (the content between the quotes), per Prettier's `print-string.js`.
    ///
    /// - `json` / `jsonc` / `json-stringify`: always `"`
    /// - `json5` with `quoteProps: "preserve"` and `singleQuote: false`:
    ///   - pinned to `"` (this lets the `json5` parser double as "JSON with comments and trailing commas")
    /// - `json5` otherwise: Prettier's `getPreferredQuote`
    ///   - start from the configured preference (`singleQuote`) and flip to the alternate when that reduces escapes
    ///   - (i.e. when the preferred quote occurs more often in `inner` than the alternate)
    pub fn preferred_quote(&self, inner: &str) -> u8 {
        // Non-json5 always double-quotes: bail before touching any json5-only option.
        if self.variant != JsonVariant::Json5 {
            return b'"';
        }
        let is_single_quote = self.single_quote.value();
        // `quoteProps: "preserve"` + `singleQuote: false` pins to `"` (JSON-with-comments mode).
        if matches!(self.quote_props, QuoteProps::Preserve) && !is_single_quote {
            return b'"';
        }

        let (preferred, alternate) = if is_single_quote { (b'\'', b'"') } else { (b'"', b'\'') };
        // Count every occurrence (escaped ones included, matching `getPreferredQuote`).
        let (mut preferred_count, mut alternate_count) = (0u32, 0u32);
        for byte in inner.bytes() {
            if byte == preferred {
                preferred_count += 1;
            } else if byte == alternate {
                alternate_count += 1;
            }
        }

        if preferred_count > alternate_count { alternate } else { preferred }
    }
}

/// Whether and when to quote object keys.
/// Mirrors Prettier's `quoteProps`: when (and whether) object keys keep their quotes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteProps {
    /// `quoteProps: "as-needed"`.
    /// Drop quotes from keys that are valid identifier names,
    /// quote the rest (numeric-string keys like `"1.5"` stay quoted in json5).
    #[default]
    AsNeeded,
    /// `quoteProps: "preserve"`.
    /// Keep keys exactly as authored (quoted stays quoted, unquoted stays unquoted).
    Preserve,
    /// `quoteProps: "consistent"`.
    /// If any key in an object requires quotes, quote every quotable key in that object; otherwise behave like `AsNeeded`.
    Consistent,
}

/// Whether string literals/keys prefer single quotes (`'`) over double (`"`).
/// Mirrors Prettier's `singleQuote` (default `false`).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SingleQuote(bool);

impl SingleQuote {
    pub fn value(self) -> bool {
        self.0
    }
}

impl From<bool> for SingleQuote {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Whether to insert spaces around brackets in object.
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

/// Whether objects keep their authored multi-line shape or collapse to one line when they fit.
/// Mirrors Prettier's `objectWrap` option.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Expand {
    /// `objectWrap: "preserve"`. Stays multi-line when the source has a newline right after `{`.
    #[default]
    Auto,
    /// `objectWrap: "collapse"`. Collapses when it fits regardless of authored shape.
    Never,
}

/// Whether to print a trailing comma after the last entry of a multi-line object/array.
///
/// Mirrors Prettier's `trailingComma`, but JSON only has two meaningful states:
/// the `all`/`es5` distinction is dead for JSON (no constructs beyond ES5), so they collapse into `Always`.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Trailing comma where valid (objects, arrays). Maps from Prettier `all`/`es5`.
    /// `Always` keeps Prettier's `all` default; `Never` would drop trailing commas.
    #[default]
    Always,
    /// No trailing comma. Maps from Prettier `none`.
    Never,
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

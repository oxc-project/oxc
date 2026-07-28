use oxc_formatter_core::{
    FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// Format options for YAML.
///
/// Prettier's `yaml` language consumes the shared layout options plus
/// `proseWrap`, `singleQuote`, `bracketSpacing`, and `trailingComma`
/// (`trailingComma` is consumed by the flow-collection printer).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct YamlFormatOptions {
    /// NOTE: Present to satisfy [`FormatOptions`], but a no-op for output: YAML forbids tab indentation.
    /// The printer's indent char is decided by this field but no indent is ever emitted.
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    /// How multi-line scalars are re-flowed. Mirrors Prettier's `proseWrap`.
    pub prose_wrap: ProseWrap,
    /// Preferred quote for re-quotable scalars. Mirrors Prettier's `singleQuote`.
    pub single_quote: SingleQuote,
    /// Spaces inside `{ ... }` of flow mappings. Mirrors Prettier's `bracketSpacing`.
    pub bracket_spacing: BracketSpacing,
    /// Trailing comma in broken flow collections. Mirrors Prettier's `trailingComma`
    pub trailing_commas: TrailingCommas,
}

/// How multi-line flow scalars and folded block scalars are re-flowed.
/// Mirrors Prettier's `proseWrap`; block literals (`|`) are never re-flowed.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ProseWrap {
    /// Keep the source's line structure (default).
    #[default]
    Preserve,
    /// Fold lines to fit the print width.
    Always,
    /// Collapse each paragraph onto a single line.
    Never,
}

/// Whether `'` is the preferred quote when a scalar can be re-quoted freely.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SingleQuote(bool);

impl SingleQuote {
    /// The preferred quote as a printable token (`"'"` / `"\""`).
    pub fn as_str(self) -> &'static str {
        if self.0 { "'" } else { "\"" }
    }
}

impl From<bool> for SingleQuote {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Whether to insert spaces around brackets in flow mappings.
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

/// Whether a broken flow collection gets a trailing comma.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Prettier `trailingComma: "es5" | "all"` (default).
    #[default]
    Always,
    /// Prettier `trailingComma: "none"`.
    Never,
}

impl YamlFormatOptions {
    /// Whether a trailing comma may follow the last entry of a broken flow collection.
    pub fn allow_trailing_comma(self) -> bool {
        matches!(self.trailing_commas, TrailingCommas::Always)
    }
}

impl FormatOptions for YamlFormatOptions {
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

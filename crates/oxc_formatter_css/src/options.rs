use oxc_formatter_core::{
    FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// CSS dialect variant.
///
/// Mirrors Prettier's `css` / `scss` / `less` parsers.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CssVariant {
    /// Prettier's `parser: css` equivalent.
    #[default]
    Css,
    /// Prettier's `parser: scss` equivalent.
    /// `//` comments, `$var`, maps, control directives, the module system.
    Scss,
    /// Prettier's `parser: less` equivalent.
    /// `//` comments, `@var`, mixins, guards, detached rulesets.
    Less,
}

impl CssVariant {
    pub(crate) fn to_raffia(self) -> raffia::Syntax {
        match self {
            Self::Css => raffia::Syntax::Css,
            Self::Scss => raffia::Syntax::Scss,
            Self::Less => raffia::Syntax::Less,
        }
    }

    pub fn is_scss(self) -> bool {
        matches!(self, Self::Scss)
    }

    pub fn is_less(self) -> bool {
        matches!(self, Self::Less)
    }
}

/// Format options for CSS/SCSS/Less.
///
/// Prettier's CSS languages consume the shared layout options plus
/// `singleQuote` and `trailingComma` (SCSS maps only).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CssFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    pub variant: CssVariant,
    /// Prefer single quotes for strings. Mirrors Prettier's `singleQuote`.
    pub single_quote: SingleQuote,
    // Used by: SCSS (maps only)
    pub trailing_commas: TrailingCommas,
}

impl CssFormatOptions {
    /// Whether a trailing comma may follow the last item of a multi-line
    /// SCSS map, per [`Self::trailing_commas`].
    pub fn allow_trailing_comma(self) -> bool {
        matches!(self.trailing_commas, TrailingCommas::Always)
    }
}

/// Whether string literals prefer single quotes (`'`) over double (`"`).
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

/// Whether to print a trailing comma after the last item of a multi-line
/// SCSS map (the only CSS construct Prettier's `trailingComma` reaches).
///
/// Mirrors Prettier's `trailingComma`, but the `all`/`es5` distinction is
/// dead for CSS (`shouldPrintTrailingComma` only checks "not none"),
/// so both collapse into `Always`.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Trailing comma where valid. Maps from Prettier `all`/`es5`.
    #[default]
    Always,
    /// No trailing comma. Maps from Prettier `none`.
    Never,
}

impl FormatOptions for CssFormatOptions {
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

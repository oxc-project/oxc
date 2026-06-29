use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_formatter_core::{
    FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, PrinterOptions,
};

/// CSS dialect variant.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CssVariant {
    /// Prettier's `parser: css` equivalent.
    #[default]
    Css,
    /// Prettier's `parser: scss` equivalent.
    Scss,
    /// Prettier's `parser: less` equivalent.
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
    // Used by: CSS, SCSS, Less
    pub single_quote: SingleQuote,
    // Used by: SCSS
    pub trailing_commas: TrailingCommas,
    // Used by: CSS, SCSS, Less
    //
    // NOTE: Only the activation bit lives here.
    // The detailed Tailwind settings (config|stylesheet path, preserve-whitespace|duplicates, etc) are consumed by
    // the host-supplied sorter (`prettier-plugin-tailwindcss/sorter` on the JS side)
    // and travel separately via the host(Oxfmt)'s options payload, not through this struct.
    // `oxc_formatter_css` only needs to know whether to collect `@apply` classes.
    pub sort_tailwindcss: bool,
}

impl CssFormatOptions {
    /// Whether a trailing comma may follow the last item of a multi-line
    /// SCSS map, per [`Self::trailing_commas`].
    pub fn allow_trailing_comma(self) -> bool {
        matches!(self.trailing_commas, TrailingCommas::Always)
    }

    /// The quote byte (`b'"'` / `b'\''`) to enclose a string literal whose body is `inner`
    /// (the content between the quotes), per Prettier's `getPreferredQuote`:
    /// start from the configured preference (`singleQuote`) and flip to the alternate
    /// when that reduces escapes (i.e. when the preferred quote occurs more often in `inner` than the alternate).
    pub fn preferred_quote(&self, inner: &str) -> u8 {
        let (preferred, alternate) =
            if self.single_quote.value() { (b'\'', b'"') } else { (b'"', b'\'') };
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

/// Whether string literals prefer single quotes (`'`) over double (`"`).
/// Mirrors Prettier's `singleQuote` (default `false`).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SingleQuote(bool);

impl SingleQuote {
    pub fn value(self) -> bool {
        self.0
    }

    pub fn as_char(self) -> char {
        if self.0 { '\'' } else { '"' }
    }

    pub fn as_str(self) -> &'static str {
        if self.0 { "'" } else { "\"" }
    }

    /// Prettier's `adjustStrings` for a single token:
    /// if `token` contains only the alternate quote and not the preferred one,
    /// replace alternates with preferreds.
    /// Returns the slice borrowed when no rewrite is needed.
    pub fn requote(self, token: &str) -> Cow<'_, str> {
        let (preferred, other) = if self.0 { ('\'', '"') } else { ('"', '\'') };
        if !token.contains(other) || token.contains(preferred) {
            return Cow::Borrowed(token);
        }
        token.cow_replace(other, preferred.encode_utf8(&mut [0; 4]))
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

use std::str::FromStr;

/// Prettier Options
///
/// References
/// * <https://prettier.io/docs/en/options>
/// * <https://github.com/prettier/prettier/blob/3.3.3/src/main/core-options.evaluate.js>
/// * <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/options.js>
#[derive(Debug, Clone, Copy)]
pub struct PrettierOptions {
    /* Global Options */
    /// Specify the line length that the printer will wrap on.
    /// Default: 80
    pub print_width: usize,

    /// Specify the number of spaces per indentation-level.
    /// Default: 2
    pub tab_width: usize,

    /// Indent lines with tabs instead of spaces.
    pub use_tabs: bool,

    /// Which end of line characters to apply.
    /// Default: "lf"
    pub end_of_line: EndOfLine,

    /* JavaScript Options */
    /// Print semicolons at the ends of statements.
    /// Default: true
    pub semi: bool,

    /// Use single quotes instead of double quotes.
    /// Default: false
    pub single_quote: bool,

    /// Change when properties in objects are quoted.
    /// Default: [QuoteProps::AsNeeded]
    pub quote_props: QuoteProps,

    /// Use single quotes instead of double quotes in JSX.
    /// Default: false
    pub jsx_single_quote: bool,

    /// Print trailing commas wherever possible in multi-line comma-separated syntactic
    /// structures. (A single-line array, for example, never gets trailing commas.)
    /// Default: [TrailingComma::All]
    pub trailing_comma: TrailingComma,

    /// Print spaces between brackets in object literals.
    ///
    /// * true - Example: `{ foo: bar }`.
    /// * false - Example: `{foo: bar}`.
    ///
    /// Default: true
    pub bracket_spacing: bool,

    /// Put the `>` of a multi-line HTML (HTML, JSX) element at the end of the last line
    /// instead of being alone on the next line (does not apply to self closing elements).
    /// Default: false
    pub bracket_same_line: bool,

    /// Include parentheses around a sole arrow function parameter.
    /// Default: [ArrowParens::Always]
    pub arrow_parens: ArrowParens,
}

impl Default for PrettierOptions {
    fn default() -> Self {
        Self {
            print_width: 80,
            tab_width: 2,
            use_tabs: false,
            end_of_line: EndOfLine::default(),
            semi: true,
            single_quote: false,
            quote_props: QuoteProps::default(),
            jsx_single_quote: false,
            trailing_comma: TrailingComma::default(),
            bracket_spacing: true,
            bracket_same_line: false,
            arrow_parens: ArrowParens::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum EndOfLine {
    #[default]
    Lf,
    Crlf,
    Cr,
    Auto,
}

impl EndOfLine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
            Self::Lf | Self::Auto => "\n",
        }
    }
}

impl FromStr for EndOfLine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "crlf" => Self::Crlf,
            "cr" => Self::Cr,
            "auto" => Self::Auto,
            "lf" => Self::Lf,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum QuoteProps {
    /// Only add quotes around object properties where required.
    #[default]
    AsNeeded,
    /// If at least one property in an object requires quotes, quote all properties.
    Consistent,
    /// Respect the input use of quotes in object properties.
    Preserve,
}

impl QuoteProps {
    pub fn as_needed(self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub fn preserve(self) -> bool {
        matches!(self, Self::Preserve)
    }

    pub fn consistent(self) -> bool {
        matches!(self, Self::Consistent)
    }
}

impl FromStr for QuoteProps {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "as_needed" => Self::AsNeeded,
            "consistent" => Self::Consistent,
            "preserve" => Self::Preserve,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum TrailingComma {
    /// Trailing commas wherever possible (including function parameters and calls).
    #[default]
    All,
    /// Trailing commas where valid in ES5 (objects, arrays, etc.). Trailing commas in type parameters in TypeScript.
    ES5,
    /// No trailing commas.
    None,
}

impl TrailingComma {
    pub fn is_all(self) -> bool {
        self == Self::All
    }

    pub fn is_es5(self) -> bool {
        self == Self::ES5
    }

    pub fn is_none(self) -> bool {
        self == Self::None
    }
}

impl FromStr for TrailingComma {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "all" => Self::All,
            "es5" => Self::ES5,
            "none" => Self::None,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ArrowParens {
    /// Always include parens. `Example: (x) => x`
    #[default]
    Always,
    /// Omit parens when possible. `Example: x => x`
    Avoid,
}

impl ArrowParens {
    pub fn is_always(self) -> bool {
        matches!(self, Self::Always)
    }
}

impl FromStr for ArrowParens {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "always" => Self::Always,
            "avoid" => Self::Avoid,
            _ => Self::default(),
        })
    }
}

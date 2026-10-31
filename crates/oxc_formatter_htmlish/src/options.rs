use oxc_formatter_core::{
    CoreFormatOptions, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
};
use oxc_htmlish_parser::Dialect;

/// HTML-based template language variant.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum HtmlishVariant {
    /// Prettier's `parser: html` equivalent.
    #[default]
    Html,
    /// Prettier's `parser: vue` equivalent.
    Vue,
    /// Prettier's `parser: angular` equivalent.
    Angular,
    /// Prettier's `parser: html` on an XHTML file (`.xhtml`, `.xht`), which an XML parser may read:
    /// read and printed as HTML, but the doctype keeps its case.
    Xhtml,
}

impl HtmlishVariant {
    pub(crate) fn to_dialect(self) -> Dialect {
        match self {
            Self::Html | Self::Xhtml => Dialect::Html,
            Self::Vue => Dialect::Vue,
            Self::Angular => Dialect::Angular,
        }
    }
}

/// Format options for htmlish (HTML and the template languages built on it).
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct HtmlishFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    pub variant: HtmlishVariant,
    /// Mirrors Prettier's `htmlWhitespaceSensitivity`.
    pub whitespace_sensitivity: WhitespaceSensitivity,
    /// Put the `>` of a multi-line start tag at the end of the last line. Mirrors Prettier's `bracketSameLine`.
    pub bracket_same_line: bool,
    /// Mirrors Prettier's `singleAttributePerLine`.
    pub single_attribute_per_line: bool,
    /// Sort Tailwind CSS classes (by the session's sorter, as prettier-plugin-tailwindcss does):
    /// those of `class`, and of the attributes listed here; `None` leaves them as they are.
    pub sort_tailwindcss: Option<Vec<String>>,
}

/// Which whitespace between elements is significant.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WhitespaceSensitivity {
    /// As the element's default CSS `display` says.
    #[default]
    Css,
    /// Every element is inline: all whitespace is significant.
    Strict,
    /// Every element is a block: no whitespace is significant.
    Ignore,
}

impl FormatOptions for HtmlishFormatOptions {
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

    fn apply_core(&mut self, core: CoreFormatOptions) {
        self.indent_style = core.indent_style;
        self.indent_width = core.indent_width;
        self.line_width = core.line_width;
        self.line_ending = core.line_ending;
    }
}

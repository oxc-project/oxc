use oxc_formatter_core::{
    CoreFormatOptions, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
};

/// Format options for TOML.
///
/// Mirrors `prettier-plugin-toml`, which maps the shared layout options and `trailingComma` only.
/// <https://github.com/un-ts/prettier/blob/7a4346d5dbf6b63987c0f81228fc46bb12f8692f/packages/toml/src/index.ts#L27-L31>
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct TomlFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    pub trailing_commas: TrailingCommas,
}

/// Whether a multi-line array gets a trailing comma (TOML 1.0 allows none in inline tables).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Prettier `trailingComma: "es5" | "all"` (default).
    #[default]
    Always,
    /// Prettier `trailingComma: "none"`.
    Never,
}

impl FormatOptions for TomlFormatOptions {
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

impl TomlFormatOptions {
    pub(crate) fn to_oxc_toml(self) -> oxc_toml::Options {
        oxc_toml::Options {
            column_width: self.line_width.value() as usize,
            indent_string: if self.indent_style.is_tab() {
                "\t".to_string()
            } else {
                " ".repeat(self.indent_width.value() as usize)
            },
            crlf: self.line_ending.is_carriage_return_line_feed(),
            array_trailing_comma: matches!(self.trailing_commas, TrailingCommas::Always),
            // Callers trim it for `insertFinalNewline: false`
            trailing_newline: true,
            ..Default::default()
        }
    }
}

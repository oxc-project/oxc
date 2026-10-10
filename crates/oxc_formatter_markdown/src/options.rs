use oxc_formatter_core::{
    CoreFormatOptions, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
};

/// Format options for Markdown.
///
/// Prettier's `markdown` language consumes the shared layout options plus
/// `proseWrap` and `singleQuote` (link / image / definition titles).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MarkdownFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
    /// How paragraph text is re-flowed. Mirrors Prettier's `proseWrap`.
    pub prose_wrap: ProseWrap,
    /// Preferred quote for re-quotable titles. Mirrors Prettier's `singleQuote`.
    pub single_quote: SingleQuote,
}

/// How paragraph text is re-flowed.
/// Mirrors Prettier's `proseWrap`.
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

/// Whether `'` is the preferred quote when a title can be re-quoted freely.
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

impl MarkdownFormatOptions {
    /// The quote byte (`b'"'` / `b'\''`) to enclose a title whose body is `inner`:
    /// start from the configured preference (`singleQuote`) and flip to the alternate when that reduces escapes
    /// (i.e. when the preferred quote occurs more often in `inner` than the alternate).
    pub fn preferred_quote(self, inner: &str) -> u8 {
        let (preferred, alternate) =
            if self.single_quote.value() { (b'\'', b'"') } else { (b'"', b'\'') };
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

impl FormatOptions for MarkdownFormatOptions {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    /// Used to adjust Span positions to fit the global source code.
    pub span_offset: u32,
    /// Unicode mode(`u` or `v` flag) enabled or not.
    pub unicode_mode: bool,
    /// Extended Unicode mode(`v` flag) enabled or not.
    pub unicode_sets_mode: bool,
    // TODO: Add `handle_escape_with_quote_type` like option to support `new RegExp("with \"escape\"")`
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> Self {
        ParserOptions { span_offset, ..self }
    }

    #[must_use]
    pub fn with_flags(self, flags: &str) -> Self {
        let (mut unicode_mode, mut unicode_sets_mode) = (false, false);
        for ch in flags.chars() {
            if ch == 'u' {
                unicode_mode = true;
            }
            if ch == 'v' {
                unicode_mode = true;
                unicode_sets_mode = true;
            }
        }

        ParserOptions { unicode_mode, unicode_sets_mode, ..self }
    }
}

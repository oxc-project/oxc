use oxc_ast::ast::{Comment, RegExpFlags};
use oxc_span::Span;

use crate::{
    error::Diagnostic,
    pipeline::Lanes,
    token::{SPAN_SENTINELS, StringSpan, TokenKind},
};

#[derive(Debug, Default)]
pub struct Tokens {
    pub(crate) kinds: Vec<TokenKind>,
    pub(crate) spans: Vec<Span>,
    pub(crate) lanes: Lanes,
}

impl Tokens {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token_count(&self) -> usize {
        self.kinds.len().saturating_sub(SPAN_SENTINELS)
    }

    /// Token kinds, followed by SPAN_SENTINELS Eof entries.
    pub fn kinds(&self) -> &[TokenKind] {
        &self.kinds
    }

    /// Token spans, followed by SPAN_SENTINELS empty spans at the end of the source.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.lanes.diags
    }

    /// Values of the Number tokens, in token order.
    pub fn numbers(&self) -> &[f64] {
        &self.lanes.numbers
    }

    /// BigInt values in token order, as decimal digits in the cooked bytes without `_` or `n`.
    pub fn bigints(&self) -> &[StringSpan] {
        &self.lanes.bigints
    }

    pub fn strings(&self) -> &[StringSpan] {
        &self.lanes.strings
    }

    pub fn cooked_bytes(&self) -> &[u8] {
        &self.lanes.cooked
    }

    pub fn templates(&self) -> &[StringSpan] {
        &self.lanes.templates
    }

    /// Escaped identifier values in token order, in the cooked bytes (private names without `#`).
    /// An invalid escape adds nothing, so a value is UTF-8 whenever the source is.
    pub fn atoms(&self) -> &[StringSpan] {
        &self.lanes.atoms
    }

    pub fn regex_flags(&self) -> &[RegExpFlags] {
        &self.lanes.regex_flags
    }

    pub fn comments(&self) -> &[Comment] {
        &self.lanes.comments
    }

    pub fn irregular_whitespaces(&self) -> &[Span] {
        &self.lanes.irregular_whitespaces
    }
}

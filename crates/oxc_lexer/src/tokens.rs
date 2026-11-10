use oxc_ast::ast::{Comment, RegExpFlags};
use oxc_span::Span;

use crate::{
    error::Diagnostic,
    pipeline::TokenLanes,
    token::{SPAN_SENTINELS, StringSpan, TokenKind},
};

#[derive(Debug, Default)]
pub struct Tokens {
    pub(crate) kinds: Vec<TokenKind>,
    pub(crate) spans: Vec<Span>,
    pub(crate) lanes: TokenLanes,
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

    pub fn numbers(&self) -> &[f64] {
        &self.lanes.numbers
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

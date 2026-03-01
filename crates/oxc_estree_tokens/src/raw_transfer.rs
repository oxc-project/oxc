//! Functions and data structures to convert tokens to ESTree in place.

use std::slice::IterMut;

use oxc_ast::ast::{
    JSXText, PrivateIdentifier, Program, RegExpLiteral, StringLiteral, TemplateElement,
};
use oxc_ast_visit::{
    Visit,
    utf8_to_utf16::{Utf8ToUtf16, Utf8ToUtf16Converter},
};
use oxc_parser::{Kind, Token};

use crate::{context::Context, options::ESTreeTokenConfig, visitor::Visitor};

/// Walk AST and set token kinds to match ESTree token types.
/// Convert token spans from UTF-8 byte offsets to UTF-16 offsets.
///
/// After this pass, `get_token_type(token.kind())` returns the correct ESTree token type
/// for every token, without needing AST context.
pub fn update_tokens<O: ESTreeTokenConfig>(
    tokens: &mut [Token],
    program: &Program<'_>,
    span_converter: &Utf8ToUtf16,
    options: O,
) {
    let mut visitor = Visitor::new(RawContext {
        tokens: tokens.iter_mut(),
        span_converter: span_converter.converter(),
        options,
        jsx_state: O::JSXState::default(),
    });
    visitor.visit_program(program);
    visitor.into_ctx().finish();
}

/// Raw transfer context.
///
/// Sets token kinds so that `get_token_type(token.kind())` returns
/// the correct ESTree token type without needing AST context.
/// Also converts token spans from UTF-8 byte offsets to UTF-16 offsets.
pub struct RawContext<'b, O: ESTreeTokenConfig> {
    /// Mutable tokens iterator
    tokens: IterMut<'b, Token>,
    /// Span converter for UTF-8 to UTF-16 conversion.
    /// `None` if source is ASCII-only.
    span_converter: Option<Utf8ToUtf16Converter<'b>>,
    /// Options controlling JS/TS style differences
    options: O,
    /// JSX state tracking
    jsx_state: O::JSXState,
}

impl<O: ESTreeTokenConfig> RawContext<'_, O> {
    /// Advance iterator to the token at `start`, converting spans along the way.
    /// Skipped tokens are not modified (they already have the correct kind),
    /// but their spans are converted from UTF-8 to UTF-16.
    fn advance_to(&mut self, start: u32) -> &mut Token {
        let Self { tokens, span_converter, .. } = self;
        for token in &mut *tokens {
            debug_assert!(
                token.start() <= start,
                "Expected token at position {start}, found token at position {}",
                token.start(),
            );

            let is_target = token.start() == start;

            // Convert span from UTF-8 byte offsets to UTF-16 offsets
            if let Some(converter) = span_converter {
                let mut span = token.span();
                converter.convert_span(&mut span);
                token.set_span(span);
            }

            if is_target {
                return token;
            }
        }
        unreachable!("Expected token at position {start}");
    }

    /// Advance to the token at `start` and set its `Kind`.
    fn set_kind_at(&mut self, start: u32, kind: Kind) {
        let token = self.advance_to(start);
        token.set_kind(kind);
    }

    /// Convert remaining token spans from UTF-8 byte offsets to UTF-16 offsets.
    fn finish(self) {
        if let Some(mut converter) = self.span_converter {
            for token in self.tokens {
                let mut span = token.span();
                converter.convert_span(&mut span);
                token.set_span(span);
            }
        }
    }
}

impl<O: ESTreeTokenConfig> Context for RawContext<'_, O> {
    /// JSX state type for tracking when to emit JSX identifiers.
    /// Inherited from config.
    type JSXState = O::JSXState;

    /// Returns `true` if serializing in TS style.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_ts(&self) -> bool {
        self.options.is_ts()
    }

    /// Get reference to [`JSXState`].
    ///
    /// [`JSXState`]: crate::jsx_state::JSXState
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state(&self) -> &Self::JSXState {
        &self.jsx_state
    }

    /// Get mutable reference to [`JSXState`].
    ///
    /// [`JSXState`]: crate::jsx_state::JSXState
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state_mut(&mut self) -> &mut Self::JSXState {
        &mut self.jsx_state
    }

    /// Set `Kind` of the token at `start` to `Identifier`.
    /// In JS mode, if it's a `yield`, `let`, or `static` keyword, leave it as a `Keyword` token.
    fn emit_identifier_at(&mut self, start: u32, _name: &str) {
        let is_ts = self.is_ts();

        let token = self.advance_to(start);

        // In JS style, `yield` / `let` / `static` used as identifiers should remain as keywords
        if is_ts || !matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static) {
            token.set_kind(Kind::Ident);
        }
    }

    /// Set `Kind` of the token at `start` to `Identifier`.
    fn emit_this_identifier_at(&mut self, start: u32) {
        self.set_kind_at(start, Kind::Ident);
    }

    /// Set `Kind` of the token at `start` to `JSXIdentifier`.
    fn emit_jsx_identifier_at(&mut self, start: u32, _name: &str) {
        self.set_kind_at(start, Kind::JSXIdentifier);
    }

    /// Handle `PrivateIdentifier` token (no-op).
    #[inline(always)]
    fn emit_private_identifier(&mut self, _ident: &PrivateIdentifier<'_>) {
        // No-op: token already has `Kind::PrivateIdentifier`.
        // The iterator will skip past this token on the next `advance_to` call.
    }

    /// Handle `StringLiteral` token (no-op).
    #[inline(always)]
    fn emit_string_literal(&mut self, _literal: &StringLiteral<'_>) {
        // No-op: token already has `Kind::Str`.
        // The iterator will skip past this token on the next `advance_to` call.
    }

    /// Set `Kind` of the `StringLiteral` token to `JSXText`.
    fn emit_string_literal_as_jsx_text(&mut self, literal: &StringLiteral<'_>) {
        self.set_kind_at(literal.span.start, Kind::JSXText);
    }

    /// Handle `JSXText` token (no-op).
    #[inline(always)]
    fn emit_jsx_text(&mut self, _jsx_text: &JSXText<'_>) {
        // No-op: token already has `Kind::JSXText`.
        // The iterator will skip past this token on the next `advance_to` call.
    }

    /// Handle `RegExpLiteral` (no-op).
    #[inline(always)]
    fn emit_regexp(&mut self, _regexp: &RegExpLiteral<'_>) {
        // No-op: token already has `Kind::RegExp`.
        // The iterator will skip past this token on the next `advance_to` call.
    }

    /// Walk template interpolations (expressions or TS types).
    fn walk_template_quasis_interleaved<I>(
        visitor: &mut Visitor<Self>,
        _quasis: &[TemplateElement<'_>],
        mut visit_interpolation: impl FnMut(&mut Visitor<Self>, &I),
        interpolations: &[I],
    ) {
        // Quasis don't need kind changes, so skip them and only visit interpolations
        for interpolation in interpolations {
            visit_interpolation(visitor, interpolation);
        }
    }
}

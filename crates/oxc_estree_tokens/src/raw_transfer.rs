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

use crate::{
    context::Context, estree_kind::ESTreeKind, options::ESTreeTokenConfig, visitor::Visitor,
};

/// Walk AST and convert all token kinds to `ESTreeKind` discriminants.
/// Also convert token spans from UTF-8 byte offsets to UTF-16 offsets.
///
/// After this pass, each token's kind byte contains an `ESTreeKind` discriminant
/// that JS can use directly to look up the ESTree token type string.
///
/// After this call, `Token::kind` on the passed-in `tokens` will NOT be accurate.
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
/// Converts all token kinds to [`ESTreeKind`] discriminants and
/// token spans from UTF-8 byte offsets to UTF-16 offsets.
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
    /// Advance iterator to the token at `start`, converting spans and kinds along the way.
    ///
    /// Skipped tokens have their kinds converted to [`ESTreeKind`] via the conversion table.
    /// The target token is returned without kind conversion — the caller sets its kind explicitly.
    /// The target token *does* have its span converted from UTF-8 to UTF-16.
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

            // Return target token without kind conversion
            if is_target {
                return token;
            }

            // Convert kind
            let estree_kind = ESTreeKind::from_kind(token.kind());
            token.set_kind(estree_kind.to_kind());
        }
        unreachable!("Expected token at position {start}");
    }

    /// Advance to the token at `start` and set its [`ESTreeKind`].
    fn set_kind_at(&mut self, start: u32, estree_kind: ESTreeKind) {
        let token = self.advance_to(start);
        token.set_kind(estree_kind.to_kind());
    }

    /// Convert remaining token spans and kinds.
    fn finish(self) {
        if let Some(mut converter) = self.span_converter {
            for token in self.tokens {
                let mut span = token.span();
                converter.convert_span(&mut span);
                token.set_span(span);

                let estree_kind = ESTreeKind::from_kind(token.kind());
                token.set_kind(estree_kind.to_kind());
            }
        } else {
            for token in self.tokens {
                let estree_kind = ESTreeKind::from_kind(token.kind());
                token.set_kind(estree_kind.to_kind());
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

    /// Set token at `start` to `Identifier`.
    /// In JS mode, if it's a `yield`, `let`, or `static` keyword, set it to `Keyword` instead.
    fn emit_identifier_at(&mut self, start: u32, _name: &str) {
        let is_ts = self.is_ts();

        let token = self.advance_to(start);

        let estree_kind =
            if is_ts || !matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static) {
                ESTreeKind::Identifier
            } else {
                ESTreeKind::Keyword
            };
        token.set_kind(estree_kind.to_kind());
    }

    /// Set token at `start` to `Identifier`.
    fn emit_this_identifier_at(&mut self, start: u32) {
        self.set_kind_at(start, ESTreeKind::Identifier);
    }

    /// Set token at `start` to `JSXIdentifier`.
    fn emit_jsx_identifier_at(&mut self, start: u32, _name: &str) {
        self.set_kind_at(start, ESTreeKind::JSXIdentifier);
    }

    /// Handle `PrivateIdentifier` token (no-op).
    /// Kind is converted to `ESTreeKind::PrivateIdentifier` when skipped by `advance_to`.
    #[inline(always)]
    fn emit_private_identifier(&mut self, _ident: &PrivateIdentifier<'_>) {}

    /// Handle `StringLiteral` token (no-op).
    /// Kind is converted to `ESTreeKind::String` when skipped by `advance_to`.
    #[inline(always)]
    fn emit_string_literal(&mut self, _literal: &StringLiteral<'_>) {}

    /// Set `StringLiteral` token to `JSXText`.
    fn emit_string_literal_as_jsx_text(&mut self, literal: &StringLiteral<'_>) {
        self.set_kind_at(literal.span.start, ESTreeKind::JSXText);
    }

    /// Handle `JSXText` token (no-op).
    /// Kind is converted to `ESTreeKind::JSXText` when skipped by `advance_to`.
    #[inline(always)]
    fn emit_jsx_text(&mut self, _jsx_text: &JSXText<'_>) {}

    /// Handle `RegExpLiteral` (no-op).
    /// Kind is converted to `ESTreeKind::RegularExpression` when skipped by `advance_to`.
    #[inline(always)]
    fn emit_regexp(&mut self, _regexp: &RegExpLiteral<'_>) {}

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

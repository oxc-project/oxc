//! [`Context`] trait.

use oxc_ast::ast::{RegExpLiteral, TemplateElement};

use crate::{jsx_state::JSXState, token_type::TokenType, visitor::Visitor};

/// Trait abstracting over the two token processing modes:
/// JSON serialization ([`JsonContext`]) and raw transfer ([`RawContext`]).
///
/// Each implementation holds its own `options` and `jsx_state`, so `is_ts` / `is_js`
/// resolve statically when the generic `O: ESTreeTokenConfig` is monomorphized.
///
/// [`JsonContext`]: crate::json::JsonContext
/// [`RawContext`]: crate::raw_transfer::RawContext
pub trait Context: Sized {
    /// JSX state type for tracking when to emit JSX identifiers.
    type JSXState: JSXState;

    /// Returns `true` if serializing in TS style.
    fn is_ts(&self) -> bool;

    /// Returns `true` if serializing in JS style.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_js(&self) -> bool {
        !self.is_ts()
    }

    /// Get reference to [`JSXState`].
    fn jsx_state(&self) -> &Self::JSXState;

    /// Get mutable reference to [`JSXState`].
    fn jsx_state_mut(&mut self) -> &mut Self::JSXState;

    /// Emit the token at `start` as an identifier.
    ///
    /// * JSON mode: Serialize with type `Identifier` or `Keyword`.
    /// * Raw transfer mode: Set kind to `Kind::Ident`, unless in JS style and the token is `yield` / `let` / `static`
    ///   (which should remain as `Keyword`).
    fn emit_identifier_at(&mut self, start: u32, name: &str);

    /// Emit the `this` keyword at `start` as `Identifier`.
    ///
    /// * JSON mode: Serialize as `Identifier` / `"this"`.
    /// * Raw transfer mode: Set kind to `Kind::Ident`.
    fn emit_this_identifier_at(&mut self, start: u32);

    /// Emit the token at `start` as `JSXIdentifier`.
    ///
    /// * JSON mode: Serialize as `JSXIdentifier`.
    /// * Raw transfer mode: Set kind to `Kind::JSXIdentifier`.
    fn emit_jsx_identifier_at(&mut self, start: u32, name: &str);

    /// Emit the token at `start` as `PrivateIdentifier`.
    ///
    /// * JSON mode: Serialize with appropriate encoding.
    /// * Raw transfer mode: No-op (token already has `Kind::PrivateIdentifier`).
    fn emit_private_identifier_at(&mut self, start: u32, name: &str);

    /// Emit a `StringLiteral` in a JSX attribute as `JSXText`.
    ///
    /// Unlike [`emit_unsafe_token_at`], this changes the token's kind in raw transfer mode,
    /// because the token has `Kind::Str` but needs to become `Kind::JSXText`.
    /// Use [`emit_unsafe_token_at`] for actual `JSXText` tokens which already have the correct kind.
    ///
    /// * JSON mode: Serialize as `JSXText` with JSON encoding.
    /// * Raw transfer mode: Set kind to `Kind::JSXText`.
    ///
    /// [`emit_unsafe_token_at`]: Context::emit_unsafe_token_at
    fn emit_jsx_text_at(&mut self, start: u32);

    /// Emit a token whose value may not be JSON-safe (strings, templates, JSXText).
    ///
    /// * JSON mode: Serialize with JSON encoding.
    /// * Raw transfer mode: No-op (token already has the correct kind).
    fn emit_unsafe_token_at(&mut self, start: u32, token_type: TokenType);

    /// Emit a `RegularExpression` token.
    ///
    /// * JSON mode: Serialize using `ESTreeRegExpToken`.
    /// * Raw transfer mode: No-op (token already has the correct kind).
    fn emit_regexp(&mut self, regexp: &RegExpLiteral<'_>);

    /// Walk template quasis interleaved with their interpolated parts (expressions or TS types).
    ///
    /// * JSON mode: Emit quasi tokens interleaved with interpolation visits.
    /// * Raw transfer mode: Only visit interpolations (quasis don't need `Kind` changes).
    fn walk_template_quasis_interleaved<I>(
        visitor: &mut Visitor<Self>,
        quasis: &[TemplateElement<'_>],
        visit_interpolation: impl FnMut(&mut Visitor<Self>, &I),
        interpolations: &[I],
    );
}

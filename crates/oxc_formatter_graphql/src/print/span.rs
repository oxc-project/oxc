//! Span utilities bridging the parser AST and the formatter core.
//!
//! `oxc-graphql-parser` spans are usize-based ([`ast::Span`]) while the
//! formatter core APIs take u32-based [`oxc_span::Span`]s; [`to_span`] converts.
//! Node spans are significant-token spans (trivia is never included),
//! so layout decisions use them directly, via [`Spanned`] when generic.
//!
//! Closing-delimiter positions are derived here too:
//! - Braced/bracketed containers (`SelectionSet`, `ListValue`, ...) own their delimiter:
//!   the span's last consumed token IS the 1-byte closer, see [`close_delim_start`].
//! - Paren lists (arguments, variable definitions) are bare `AstVec`s with no wrapper node,
//!   so the `)` is found by scanning past trivia from the last item's end, see [`find_close_after`].

use oxc_graphql_parser::ast;
use oxc_span::Span;

/// Converts an `oxc-graphql-parser` [`ast::Span`] (usize-based) into
/// an [`oxc_span::Span`] (u32-based) for use with the formatter core APIs.
/// `pub`: also used by `format.rs` for the parser's comment spans (re-exported through `print`).
#[inline]
pub fn to_span(s: ast::Span) -> Span {
    Span::new(u32::try_from(s.start).unwrap_or(u32::MAX), u32::try_from(s.end).unwrap_or(u32::MAX))
}

/// Significant span of an AST node as an [`oxc_span::Span`].
pub(super) trait Spanned {
    fn span(&self) -> Span;
}

macro_rules! impl_spanned {
    ($($ty:ident),* $(,)?) => {
        $(
            impl Spanned for ast::$ty<'_> {
                fn span(&self) -> Span {
                    to_span(self.span)
                }
            }
        )*
    };
}
impl_spanned!(
    Argument,
    EnumValueDefinition,
    FieldDefinition,
    InputValueDefinition,
    ObjectField,
    RootOperationTypeDefinition,
    VariableDefinition,
);

// Enums: forward to the parser's inherent `span()` accessors.
// Explicit calls: the inherent method (returning `ast::Span`) would shadow
// the trait method under plain `self.span()`.

impl Spanned for ast::Definition<'_> {
    fn span(&self) -> Span {
        to_span(ast::Definition::span(self))
    }
}

impl Spanned for ast::Selection<'_> {
    fn span(&self) -> Span {
        to_span(ast::Selection::span(self))
    }
}

impl Spanned for ast::Value<'_> {
    fn span(&self) -> Span {
        to_span(ast::Value::span(self))
    }
}

/// Start offset of a container's 1-byte closing delimiter (`}` / `]`).
///
/// Sound only on a clean parse — the only case that reaches the printer,
/// since `format()` bails on any parse error.
/// Error-recovery spans may end elsewhere (e.g. at EOF),
/// which would misplace comments flushed against this position.
pub(super) fn close_delim_start(span: ast::Span) -> u32 {
    to_span(span).end.saturating_sub(1)
}

/// Scans `source` from `from` for the first byte equal to `close`,
/// treating GraphQL trivia (whitespace, insignificant commas, `#` line comments) as skippable.
/// Returns `from` if not found
/// (defensive fallback; the caller's parse succeeded so the closing byte is expected to exist).
pub(super) fn find_close_after(source: &str, from: u32, close: u8) -> u32 {
    let bytes = source.as_bytes();
    let mut i = from as usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == close {
            return u32::try_from(i).unwrap_or(from);
        }
        if b == b'#' {
            // Line comment: skip to end of line (matching GraphQL's line terminators)
            while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'\r' {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    from
}

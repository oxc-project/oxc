//! Span utilities bridging the parser AST and the formatter core.
//!
//! `oxc-graphql-parser` has its own span type ([`ast::Span`]) while the
//! formatter core APIs take [`oxc_span::Span`]s; [`to_span`] converts.
//! Node spans are significant-token spans (trivia is never included),
//! so layout decisions use them directly, via [`Spanned`] when generic.
//!
//! Closing-delimiter positions are derived here too:
//! delimited nodes carry spans whose last consumed token IS the 1-byte closer, see [`close_delim_start`].
//! Keyword-led productions start their spans at the keyword/punctuation itself.
//! Tokens the grammar leaves bare (`:`, `repeatable`, the directive-definition `on`, the schema braces, union `|`) have no node;
//! their positions are resolved by [`next_token_start_after`] source scans, same as `oxc_formatter` does for JS.

use oxc_graphql_parser::ast;
use oxc_span::Span;

/// Converts an `oxc-graphql-parser` [`ast::Span`] into an [`oxc_span::Span`]
/// for use with the formatter core APIs (both are u32-based since parser 0.0.5).
/// `pub`: also used by `format.rs` for the parser's comment spans (re-exported through `print`).
#[inline]
pub fn to_span(s: ast::Span) -> Span {
    Span::new(s.start, s.end)
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
    span.end.saturating_sub(1)
}

/// Start of the first significant byte at or after `from`,
/// skipping GraphQL trivia (whitespace, insignificant commas, `#` line comments).
///
/// Locates formatter-emitted literals the AST carries no node for:
/// a trailing-comment claim in front of such a literal must be bounded by the literal's SOURCE position,
/// or it would pull a comment that trails the literal itself backwards across it
/// (`g: # c` must not become `g # c` + `:`).
pub(super) fn next_token_start_after(source: &str, from: u32) -> u32 {
    let bytes = source.as_bytes();
    let mut i = from as usize;
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' | b',' => i += 1,
            b'#' => {
                while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'\r' {
                    i += 1;
                }
            }
            _ => break,
        }
    }
    u32::try_from(i).unwrap_or(from)
}

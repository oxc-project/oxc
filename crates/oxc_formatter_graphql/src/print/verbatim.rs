//! Verbatim slices: a construct whose source already reads exactly as printed
//! (no whitespace, comma or comment between its tokens beyond what the printer writes itself)
//! prints as ONE source slice instead of a text/token per piece.
//!
//! No comment can sit inside such a slice, so the claims the piecewise path makes inside it are no-ops;
//! only the leading claim at its start remains.
//!
//! Each check relies on spelling the grammar fixes (`$var`, `[Foo!]!`, literal values),
//! except the `": "` of [`write_colon_pair`], a layout choice:
//! what follows it is atomic or breaks inside its own `[` / `{`, so a break after the `:` gains no width.
//! Change both of its sides together.

use oxc_formatter_core::{builders::text, write};
use oxc_graphql_parser::ast::{self, Name, Type, Value, Variable};
use oxc_span::Span;

use crate::comments::flush_leading_comments;

use super::{
    GraphqlFormatter, common, flush_trailing_before_literal, span::to_span, string, value,
};

/// One side of a `<head>: <tail>` pair.
pub(super) trait Piece<'a>: Copy {
    fn span(self) -> Span;
    /// Whether the piece prints exactly as its source text.
    fn is_verbatim(self, _: &GraphqlFormatter<'_, 'a>) -> bool {
        true
    }
    /// Prints the piece piecewise.
    fn write(self, f: &mut GraphqlFormatter<'_, 'a>);
}

impl<'a> Piece<'a> for &Name<'a> {
    fn span(self) -> Span {
        to_span(self.span)
    }
    fn write(self, f: &mut GraphqlFormatter<'_, 'a>) {
        common::write_name(self, f);
    }
}

impl<'a> Piece<'a> for &Variable<'a> {
    fn span(self) -> Span {
        to_span(self.span)
    }
    fn is_verbatim(self, _: &GraphqlFormatter<'_, 'a>) -> bool {
        sigil_span(self.span, "$", &self.name).is_some()
    }
    fn write(self, f: &mut GraphqlFormatter<'_, 'a>) {
        common::write_variable(self, f);
    }
}

impl<'a> Piece<'a> for &Type<'a> {
    fn span(self) -> Span {
        to_span(Type::span(self))
    }
    /// Spelled exactly as printed (`[Foo!]!`).
    fn is_verbatim(self, f: &GraphqlFormatter<'_, 'a>) -> bool {
        match self {
            // A name is all `[A-Za-z0-9_]`
            Type::Named(_) => true,
            Type::Missing(_) => false,
            Type::List(_) | Type::NonNull(_) => {
                let span = self.span();
                f.context().source_text().all_bytes_match(span.start, span.end, |b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'[' | b']' | b'!')
                })
            }
        }
    }
    fn write(self, f: &mut GraphqlFormatter<'_, 'a>) {
        common::write_type(self, f);
    }
}

impl<'a> Piece<'a> for &'a Value<'a> {
    fn span(self) -> Span {
        to_span(Value::span(self))
    }
    fn is_verbatim(self, f: &GraphqlFormatter<'_, 'a>) -> bool {
        match self {
            Value::Int(_)
            | Value::Float(_)
            | Value::Boolean(_)
            | Value::Null(_)
            | Value::Enum(_) => true,
            Value::String(sv) => string::is_verbatim(sv),
            Value::Variable(variable) => variable.is_verbatim(f),
            Value::List(_) | Value::Object(_) | Value::Missing(_) => false,
        }
    }
    fn write(self, f: &mut GraphqlFormatter<'_, 'a>) {
        value::write_value(self, f);
    }
}

/// The position right after a head already written, like the `)` of a field's arguments (`): Type`).
#[derive(Clone, Copy)]
pub(super) struct Written(pub u32);

impl<'a> Piece<'a> for Written {
    fn span(self) -> Span {
        Span::empty(self.0)
    }
    fn write(self, _: &mut GraphqlFormatter<'_, 'a>) {}
}

/// `<head>: <tail>`, one source slice when both sides print as their source
/// and `": "` sits right between them.
pub(super) fn write_colon_pair<'a>(
    head: impl Piece<'a>,
    tail: Option<impl Piece<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let head_span = head.span();
    if let Some(tail) = tail {
        let tail_span = tail.span();
        if tail_span.start == head_span.end + 2
            && f.context().source_text().bytes_range(head_span.end, tail_span.start) == b": "
            && head.is_verbatim(f)
            && tail.is_verbatim(f)
        {
            flush_leading_comments(head_span.start, f);
            write_slice(Span::new(head_span.start, tail_span.end), f);
            return;
        }
    }
    head.write(f);
    flush_trailing_before_literal(head_span.end, f);
    write!(f, ": ");
    if let Some(tail) = tail {
        tail.write(f);
    }
}

/// `<sigil><name>` (`$var`, `@directive`, `...Fragment`) of the construct at `span`.
/// Claims at the construct's span start so a leading comment lands before the sigil,
/// not between the sigil and the name.
pub(super) fn write_sigil_name<'a>(
    span: ast::Span,
    sigil: &'static str,
    name: &Name<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    flush_leading_comments(to_span(span).start, f);
    if let Some(span) = sigil_span(span, sigil, name) {
        write_slice(span, f);
    } else {
        write!(f, sigil);
        common::write_name(name, f);
    }
}

pub(super) fn write_slice(span: Span, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, text(f.context().source_text().text_for(&span)));
}

/// The source span of `sigil` and `name` when the name directly follows the sigil.
fn sigil_span(span: ast::Span, sigil: &str, name: &Name<'_>) -> Option<Span> {
    let span = to_span(span);
    let name = to_span(name.span);
    ((name.start - span.start) as usize == sigil.len()).then(|| Span::new(span.start, name.end))
}

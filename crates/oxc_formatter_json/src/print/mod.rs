use std::borrow::Cow;

use oxc_ast::ast::Expression;
use oxc_formatter_core::{Buffer, Format, Formatter, builders::FormatWith, builders::text, write};
use oxc_span::{GetSpan, Span};

use crate::{
    comments::{FormatLeadingComments, FormatSuppressedNode, is_suppressed_before},
    context::JsonFormatContext,
};

pub mod array;
pub mod literal;
pub mod object;

pub type JsonFormatter<'buf, 'a> = Formatter<'buf, 'a, JsonFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `JsonFormatContext`.
///
/// Hardcoded to `JsonFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap (`str` doesn't impl `Format` for any C).
impl<'a> Format<'a, JsonFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        write!(f, oxc_formatter_core::builders::token(self));
    }
}

/// Lifts a `Cow<'a, str>` to `&'a str`, allocating in the arena only for the owned case.
/// Borrowed Cows already point into arena-resident source, so they pass through unchanged.
pub fn arena_cow_str<'a>(cow: Cow<'a, str>, f: &JsonFormatter<'_, 'a>) -> &'a str {
    match cow {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => f.allocator().alloc_str(&s),
    }
}

/// Writes `body` enclosed in the character named by `quote_byte` (`b'"'` / `b'\''`).
/// `body` must already be escape-normalized for that quote.
pub fn write_quoted_str<'a>(f: &mut JsonFormatter<'_, 'a>, quote_byte: u8, body: &'a str) {
    let quote = if quote_byte == b'\'' { "'" } else { "\"" };
    write!(f, [text(quote), text(body), text(quote)]);
}

/// Wraps a re-entrant JSON closure in a [`FormatWith`]. The closure's context is
/// pinned to [`JsonFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut JsonFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// Top-level wrapper around an [`Expression`]. Dispatches by variant.
///
/// Drains any leading comments that precede `expression.span.start` and emits them before
/// the value itself.
pub struct FmtJsonValue<'a, 'b> {
    pub expression: &'b Expression<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonValue<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        let span = self.expression.span();

        if is_suppressed_before(f, span.start) {
            write!(f, FormatSuppressedNode(span));
            return;
        }

        write!(f, FormatLeadingComments(span));

        match self.expression {
            Expression::NullLiteral(_) => write!(f, "null"),
            Expression::BooleanLiteral(lit) => {
                write!(f, if lit.value { "true" } else { "false" });
            }
            Expression::NumericLiteral(lit) => literal::FmtJsonNumber { lit }.fmt(f),
            Expression::StringLiteral(lit) => literal::FmtJsonString { lit }.fmt(f),
            Expression::ArrayExpression(arr) => {
                array::FmtJsonArray { array: arr }.fmt(f);
            }
            Expression::ObjectExpression(obj) => {
                object::FmtJsonObject { object: obj }.fmt(f);
            }
            // `-9876.54321`, `+123`, `-Infinity`, etc. Prettier's `json` parser routes
            // through the JS estree printer, which keeps both `+` and `-` operators
            // while recursing into the argument so the inner number is normalized
            // (`-1.0e+2` → `-1.0e2`).
            Expression::UnaryExpression(unary) => {
                write!(f, unary.operator.as_str());
                FmtJsonValue { expression: &unary.argument }.fmt(f);
            }
            _ => write!(f, FormatInvalidJson(span)),
        }
    }
}

/// `Format` adapter for JSON-invalid AST nodes: records a diagnostic via
/// [`JsonFormatContext::report_invalid_json`] and emits the node's source verbatim
/// via [`FormatSuppressedNode`] so IR construction can still complete.
pub struct FormatInvalidJson(pub Span);

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatInvalidJson {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        f.context().report_invalid_json(self.0);
        write!(f, FormatSuppressedNode(self.0));
    }
}

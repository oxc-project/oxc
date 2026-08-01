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
mod stringify;

pub use stringify::FmtJsonStringifyValue;

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

/// Writes `body` enclosed in the character named by `quote_byte` (`b'"'` / `b'\''`).
/// `body` must already be escape-normalized for that quote.
pub fn write_quoted_str<'a>(f: &mut JsonFormatter<'_, 'a>, quote_byte: u8, body: &'a str) {
    let quote = if quote_byte == b'\'' { "'" } else { "\"" };
    write!(f, [text(quote), text(body), text(quote)]);
}

/// Emulates JS `String(Number(s)) === s`: `true` when `s` parses as a finite `f64`
/// whose ECMAScript string form is byte-identical to `s`.
/// Both the `json`/`jsonc` and `json-stringify` numeric-key quoting rules build on this.
///
/// Rust's `f64` parsing and JS `Number()` differ on some inputs
/// (hex / binary / octal prefixes, numeric separators),
/// but every such input fails the round-trip on both sides.
/// JS stringifies the parsed value into a different shape (`"16"` for `0x10`, `"NaN"` for `1_2`),
/// Rust simply fails to parse, so `Err` mapping to `false` matches.
pub fn number_string_round_trips(s: &str) -> bool {
    // PERF: Use `dragonbox_ecma` directly instead of `oxc_syntax::ToJsString`
    // The latter returns `String` (heap alloc),
    // while `Buffer::format` returns `&str` borrowed from a stack buffer.
    // This is only a `==` comparison on a hot path (every numeric key), so avoid the alloc.
    s.parse::<f64>()
        .is_ok_and(|value| value.is_finite() && dragonbox_ecma::Buffer::new().format(value) == s)
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
            // `-9876.54321`, `+123`, `-Infinity`, etc.
            // Prettier's `json` parser routes through the JS estree printer,
            // which keeps both `+` and `-` operators while recursing into the argument
            // so the inner number is normalized (`-1.0e+2` → `-1.0e2`).
            Expression::UnaryExpression(unary) => {
                write!(f, unary.operator.as_str());
                FmtJsonValue { expression: &unary.argument }.fmt(f);
            }
            // JSON5 `Infinity` / `NaN`, JSON6 `undefined`
            // Prettier accepts these identifiers for every JSON variant and prints them verbatim.
            // Other identifiers stay invalid (Prettier rejects them at parse time, we report at format time).
            Expression::Identifier(ident)
                if matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined") =>
            {
                write!(f, text(ident.name.as_str()));
            }
            // A substitution-free template literal is kept verbatim, backticks included
            // (Prettier's shared `estree` printer emits the quasi raw;
            // only `json-stringify` converts it to a double-quoted string).
            // `raw` is safe for `text()`: the lexer normalizes `\r\n` / `\r` to `\n` (spec TRV).
            Expression::TemplateLiteral(template)
                if template.expressions.is_empty() && template.quasis.len() == 1 =>
            {
                write!(f, [text("`"), text(template.quasis[0].value.raw.as_str()), text("`")]);
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

use oxc_ast::ast::{Expression, UnaryExpression};
use oxc_formatter_core::{
    Format, FormatElement, Formatter, builders::FormatWith, builders::text, write,
};
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

/// Writes the `: ` that follows an object key.
///
/// `verbatim_key` is the span of a key that prints as its own source text and is not written yet.
/// Where the source reads `<key>: `, the key and `: ` are written as one source slice,
/// extended over `value` when it starts right after the `: ` (no comment in between)
/// and `value_prints_verbatim` (the value printer's own predicate) says it prints as its own source text.
/// This saves IR elements, the printed text is the same.
///
/// Returns `true` if `value` has been written too.
pub fn write_key_separator<'a>(
    verbatim_key: Option<Span>,
    value: &Expression<'a>,
    value_prints_verbatim: fn(&Expression<'a>, &JsonFormatter<'_, 'a>) -> bool,
    f: &mut JsonFormatter<'_, 'a>,
) -> bool {
    let Some(key_span) = verbatim_key else {
        write!(f, ": ");
        return false;
    };
    let source = f.context().source_text();
    // In bounds: a key is always followed by the `:` and a value
    let colon_end = key_span.end + 2;
    if source.bytes_range(key_span.end, colon_end) != b": " {
        write!(f, [text(source.text_for(&key_span)), ": "]);
        return false;
    }
    let value_span = value.span();
    if value_span.start == colon_end && value_prints_verbatim(value, f) {
        write!(f, text(source.slice_range(key_span.start, value_span.end)));
        return true;
    }
    write!(f, text(source.slice_range(key_span.start, colon_end)));
    false
}

/// Whether `value` is a scalar that [`FmtJsonValue`] prints as its own source text
/// (when no comment precedes it), see [`write_key_separator`].
fn scalar_prints_verbatim<'a>(value: &Expression<'a>, f: &JsonFormatter<'_, 'a>) -> bool {
    match value {
        Expression::StringLiteral(lit) => literal::string_prints_verbatim(lit, f),
        Expression::NumericLiteral(lit) => literal::number_prints_verbatim(lit),
        // The parser rejects escaped keywords, so the source is exactly `true` / `false` / `null`
        Expression::BooleanLiteral(_) | Expression::NullLiteral(_) => true,
        _ => false,
    }
}

/// Writes the static `token`, which sits at `pos` in the source.
///
/// When the last written element is a text slicing the source up to `pos`
/// and the source continues with `token` (`"a": {`, `"a": 1,`),
/// the text is extended over the token instead: the same output in one IR element less.
/// The extended text is a plain [`text`], like every source slice this crate writes.
pub fn write_glued_token(pos: u32, token: &'static str, f: &mut JsonFormatter<'_, '_>) {
    let source = f.context().source_text().as_str();
    let pos = pos as usize;
    if let Some(FormatElement::Text { text: last, .. }) = f.elements().last()
        // Ending exactly at `pos` inside the source buffer, the text is that memory:
        // no other allocation can end inside it.
        && last.as_bytes().as_ptr_range().end == source.as_ptr().wrapping_add(pos)
        && let Some(extended) =
            pos.checked_sub(last.len()).and_then(|start| source.get(start..pos + token.len()))
        && extended.ends_with(token)
    {
        let last = f.elements().len() - 1;
        f.replace_end(last, &[]);
        write!(f, text(extended));
    } else {
        write!(f, token);
    }
}

/// Writes the `,` right after an entry that ends at `entry_end`,
/// merged into the entry's last element where that prints the same in one IR element less:
/// - a closing bracket token becomes the static token with the comma (`},`)
/// - a source-slice text extends over a glued comma, see [`write_glued_token`]
pub fn write_comma_after(entry_end: u32, f: &mut JsonFormatter<'_, '_>) {
    if let Some(FormatElement::Token { text: last }) = f.elements().last()
        && let Some(with_comma) = match last.as_str() {
            "}" => Some("},"),
            "]" => Some("],"),
            _ => None,
        }
    {
        let last = f.elements().len() - 1;
        f.replace_end(last, &[FormatElement::token(with_comma)]);
    } else {
        write_glued_token(entry_end, ",", f);
    }
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
    // PERF: Use `zmij_ecma` directly instead of `oxc_syntax::ToJsString`
    // The latter returns `String` (heap alloc),
    // while `Buffer::format` returns `&str` borrowed from a stack buffer.
    // This is only a `==` comparison on a hot path (every numeric key), so avoid the alloc.
    s.parse::<f64>()
        .is_ok_and(|value| value.is_finite() && zmij_ecma::Buffer::new().format(value) == s)
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
            Expression::UnaryExpression(unary) if is_signed_number(unary) => {
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

/// Returns `true` for `+` / `-` before a number, `Infinity` or `NaN`, the only unary forms Prettier accepts.
/// Others are invalid, and printing them would change or break the source (`typeof x` -> `typeofx`, `-(-1)` -> `--1`).
fn is_signed_number(unary: &UnaryExpression<'_>) -> bool {
    unary.operator.is_arithmetic()
        && match &unary.argument {
            Expression::NumericLiteral(_) => true,
            Expression::Identifier(ident) => matches!(ident.name.as_str(), "Infinity" | "NaN"),
            _ => false,
        }
}

/// `Format` adapter for JSON-invalid AST nodes: records a diagnostic via
/// [`JsonFormatContext::report_invalid_json`] and emits the node's source verbatim
/// via [`FormatSuppressedNode`] so IR construction can still complete.
///
/// NOTE: Validation happens at format time, so a node under `prettier-ignore` / `oxfmt-ignore` is never checked
/// (e.g. `// prettier-ignore\n-true` is accepted, Prettier rejects it at parse time).
/// Accepted since it is rare and the source is kept as-is;
/// validating at parse time would need another AST walk.
pub struct FormatInvalidJson(pub Span);

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatInvalidJson {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        f.context().report_invalid_json(self.0);
        write!(f, FormatSuppressedNode(self.0));
    }
}

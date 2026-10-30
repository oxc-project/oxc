use std::borrow::Cow;

use oxc_ast::ast::{NumericLiteral, StringLiteral};
use oxc_formatter_core::{
    Format, FormatContext, arena_cow_str,
    builders::text,
    spec::{format_trimmed_number, normalize_string},
    write,
};

use oxc_span::Span;

use crate::context::JsonFormatContext;

use super::{JsonFormatter, write_quoted_str};

pub struct FmtJsonString<'a, 'b> {
    pub lit: &'b StringLiteral<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonString<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        match printed_string(self.lit, f) {
            PrintedString::Verbatim(raw) => write!(f, text(raw)),
            PrintedString::Quoted { quote, body } => {
                write_quoted_str(f, quote, arena_cow_str(&body, f));
            }
        }
    }
}

/// Writes a string literal like [`FmtJsonString`], except one that prints as its own source text:
/// that one is left unwritten and its span returned,
/// so the caller can write it as part of a longer source slice.
#[must_use]
pub fn write_string_unless_verbatim<'a>(
    lit: &StringLiteral<'a>,
    f: &mut JsonFormatter<'_, 'a>,
) -> Option<Span> {
    match printed_string(lit, f) {
        PrintedString::Verbatim(_) => Some(lit.span),
        PrintedString::Quoted { quote, body } => {
            write_quoted_str(f, quote, arena_cow_str(&body, f));
            None
        }
    }
}

/// Whether [`FmtJsonString`] prints `lit` as its own source text.
pub fn string_prints_verbatim(lit: &StringLiteral<'_>, f: &JsonFormatter<'_, '_>) -> bool {
    matches!(printed_string(lit, f), PrintedString::Verbatim(_))
}

/// How a string literal prints.
enum PrintedString<'a> {
    /// Unchanged: the literal's `raw` source text.
    Verbatim(&'a str),
    /// `body` (already escape-normalized) re-enclosed in `quote`.
    Quoted { quote: u8, body: Cow<'a, str> },
}

fn printed_string<'a>(lit: &StringLiteral<'a>, f: &JsonFormatter<'_, '_>) -> PrintedString<'a> {
    // The parser always populates `raw` (see `oxc_parser::js::expression`)
    let raw = lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`"));
    let raw_str = raw.as_str();

    // Detect the outer quote.
    // Anything unexpected (no surrounding quotes, mismatched delimiters) falls back to verbatim source.
    let Some(outer_quote) = outer_quote_of(raw_str) else {
        return PrintedString::Verbatim(raw_str);
    };
    let inner = &raw_str[1..raw_str.len() - 1];

    // `json` always normalizes to `"`, but `json5` may keep/choose `'`
    let chosen_quote = f.context().options().preferred_quote(inner);
    let quotes_will_change = outer_quote != chosen_quote;

    // Prettier normalizes string quotes to `chosen_quote` and `\r\n` / lone `\r` to `\n`.
    // The shared `normalize_string` handles both.
    let normalized = normalize_string(inner, chosen_quote, quotes_will_change);

    // Fast path: outer already matched the chosen quote and the body wasn't rewritten
    if !quotes_will_change && matches!(normalized, Cow::Borrowed(_)) {
        return PrintedString::Verbatim(raw_str);
    }

    PrintedString::Quoted { quote: chosen_quote, body: normalized }
}

/// Returns the outer quote byte of `raw` (`b'"'` / `b'\''`)
/// if it's a valid quoted-string literal (matching `'…'` or `"…"`),
/// otherwise `None`.
fn outer_quote_of(raw: &str) -> Option<u8> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 || bytes[0] != bytes[bytes.len() - 1] {
        return None;
    }
    matches!(bytes[0], b'"' | b'\'').then_some(bytes[0])
}

pub struct FmtJsonNumber<'a, 'b> {
    pub lit: &'b NumericLiteral<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonNumber<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        let normalized = printed_number(self.lit);
        write!(f, text(arena_cow_str(&normalized, f)));
    }
}

/// The text [`FmtJsonNumber`] prints for `lit`, Prettier's number normalization:
/// `.1` → `0.1`, `1.0e+2` → `1.0e2`, `1e00` → `1`, `1.00000` → `1.0`, trailing `.` removal, etc.
/// `keep_one_trailing_decimal_zero` matches Prettier's JS/JSON behavior (`1.00000` → `1.0`, not `1`).
pub fn printed_number<'a>(lit: &NumericLiteral<'a>) -> Cow<'a, str> {
    format_trimmed_number(raw_number(lit), /* keep_one_trailing_decimal_zero */ true)
}

/// Whether [`FmtJsonNumber`] prints `lit` as its own source text.
pub fn number_prints_verbatim(lit: &NumericLiteral<'_>) -> bool {
    printed_number(lit) == raw_number(lit)
}

fn raw_number<'a>(lit: &NumericLiteral<'a>) -> &'a str {
    lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`")).as_str()
}

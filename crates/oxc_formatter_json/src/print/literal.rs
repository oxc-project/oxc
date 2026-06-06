use std::borrow::Cow;

use oxc_ast::ast::{NumericLiteral, StringLiteral};
use oxc_formatter_core::{
    Buffer, Format,
    builders::text,
    spec::{format_trimmed_number, normalize_string},
    write,
};

use crate::context::JsonFormatContext;

use super::{JsonFormatter, arena_cow_str};

pub struct FmtJsonString<'a, 'b> {
    pub lit: &'b StringLiteral<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonString<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        // The parser always populates `raw` (see `oxc_parser::js::expression`)
        let raw = self.lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`"));
        let raw_str = raw.as_str();

        // Detect the outer quote.
        // Anything unexpected (no surrounding quotes, mismatched delimiters) falls back to verbatim source.
        let Some(outer_quote) = outer_quote_of(raw_str) else {
            write!(f, text(raw_str));
            return;
        };
        let inner = &raw_str[1..raw_str.len() - 1];
        let quotes_will_change = outer_quote != b'"';

        // Prettier's `json` parser normalizes string quotes to `"` and `\r\n` / lone `\r` to `\n`.
        // The shared `normalize_string` handles both.
        let normalized = normalize_string(inner, b'"', quotes_will_change);

        // Fast path: outer was already `"` and the body wasn't rewritten.
        if !quotes_will_change && matches!(normalized, Cow::Borrowed(_)) {
            write!(f, text(raw_str));
            return;
        }

        write!(f, [text("\""), text(arena_cow_str(normalized, f)), text("\"")]);
    }
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
        let raw = self.lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`"));
        // Apply Prettier's number normalization: `.1` → `0.1`, `1.0e+2` → `1.0e2`,
        // `1e00` → `1`, `1.00000` → `1.0`, trailing `.` removal, etc.
        // `keep_one_trailing_decimal_zero` matches Prettier's JS/JSON behavior (`1.00000` → `1.0`, not `1`).
        let normalized =
            format_trimmed_number(raw.as_str(), /* keep_one_trailing_decimal_zero */ true);
        write!(f, text(arena_cow_str(normalized, f)));
    }
}

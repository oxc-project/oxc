//! Dedicated printer for the `json-stringify` variant.
//!
//! Comments never reach this printer:
//! the `json-stringify` variant rejects them at parse time (see `parse::validate_comments_for_variant`),
//! which is also why none of the comment / suppression machinery appears here.

use std::borrow::Cow;

use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, NumericLiteral, ObjectExpression,
    ObjectPropertyKind, PropertyKey, StringLiteral, TemplateLiteral,
};
use oxc_formatter_core::{
    Buffer, Format, arena_cow_str,
    builders::{block_indent, hard_line_break, space, text},
    write,
};
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

use crate::context::JsonFormatContext;

use super::{
    FormatInvalidJson, JsonFormatter, format_with, literal::FmtJsonString,
    number_string_round_trips, write_quoted_str,
};

/// Top-level wrapper around an [`Expression`] for the `json-stringify` variant.
/// The counterpart of [`super::FmtJsonValue`]; recursion stays within this type.
pub struct FmtJsonStringifyValue<'a, 'b> {
    pub expression: &'b Expression<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonStringifyValue<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        match self.expression {
            Expression::NullLiteral(_) => write!(f, "null"),
            Expression::BooleanLiteral(lit) => {
                write!(f, if lit.value { "true" } else { "false" });
            }
            Expression::NumericLiteral(lit) => write_number(lit, f),
            Expression::StringLiteral(lit) => write_string(lit, f),
            Expression::ArrayExpression(array) => write_array(array, f),
            Expression::ObjectExpression(object) => write_object(object, f),
            Expression::UnaryExpression(unary) => match unary.operator {
                // `JSON.parse` rejects a leading `+`, so Prettier drops it.
                UnaryOperator::UnaryPlus => {
                    Self { expression: &unary.argument }.fmt(f);
                }
                UnaryOperator::UnaryNegation => {
                    write!(f, "-");
                    Self { expression: &unary.argument }.fmt(f);
                }
                _ => write!(f, FormatInvalidJson(unary.span)),
            },
            // JSON5 `Infinity` / `NaN`, JSON6 `undefined`; anything else is invalid
            // (Prettier rejects other identifiers at parse time, we report at format time).
            Expression::Identifier(ident)
                if matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined") =>
            {
                write!(f, text(ident.name.as_str()));
            }
            Expression::TemplateLiteral(template) => write_template(template, f),
            _ => write!(f, FormatInvalidJson(self.expression.span())),
        }
    }
}

fn write_number<'a>(lit: &NumericLiteral<'a>, f: &mut JsonFormatter<'_, 'a>) {
    let raw = lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`"));
    write!(f, text(raw.as_str()));
}

fn write_string<'a>(lit: &StringLiteral<'a>, f: &mut JsonFormatter<'_, 'a>) {
    // `preferred_quote` already pins `json-stringify` to `"`.
    FmtJsonString { lit }.fmt(f);
}

/// The always-expanded layout shared by arrays and objects: a [`block_indent`]
/// whose body is `len` entries joined by `,` + hard line break.
fn write_hard_broken_entries<'a>(
    f: &mut JsonFormatter<'_, 'a>,
    len: usize,
    write_entry: impl Fn(usize, &mut JsonFormatter<'_, 'a>),
) {
    let body = format_with(|f| {
        for i in 0..len {
            if i > 0 {
                write!(f, [",", hard_line_break()]);
            }
            write_entry(i, f);
        }
    });
    write!(f, block_indent(&body));
}

/// `[` + one element per line + `]`.
/// Holes print as `null`,
/// this matches the Babel AST's `null` elements, which Prettier's printer turns into the string `"null"`).
fn write_array<'a>(array: &ArrayExpression<'a>, f: &mut JsonFormatter<'_, 'a>) {
    write!(f, "[");
    if array.elements.is_empty() {
        write!(f, "]");
        return;
    }

    write_hard_broken_entries(f, array.elements.len(), |i, f| {
        let element = &array.elements[i];
        if matches!(element, ArrayExpressionElement::Elision(_)) {
            write!(f, "null");
        } else if let Some(expression) = element.as_expression() {
            FmtJsonStringifyValue { expression }.fmt(f);
        } else {
            // The only remaining variant (after `Elision` and `Expression`) is `SpreadElement`.
            write!(f, FormatInvalidJson(element.span()));
        }
    });
    write!(f, "]");
}

/// `{` + one `key: value` per line + `}`.
fn write_object<'a>(object: &ObjectExpression<'a>, f: &mut JsonFormatter<'_, 'a>) {
    write!(f, "{");
    if object.properties.is_empty() {
        write!(f, "}");
        return;
    }

    write_hard_broken_entries(f, object.properties.len(), |i, f| match &object.properties[i] {
        ObjectPropertyKind::ObjectProperty(prop) => {
            write_object_key(&prop.key, f);
            write!(f, [":", space()]);
            FmtJsonStringifyValue { expression: &prop.value }.fmt(f);
        }
        ObjectPropertyKind::SpreadProperty(spread) => {
            write!(f, FormatInvalidJson(spread.span));
        }
    });
    write!(f, "}");
}

/// Object-key printing per Prettier's `estree-json`:
/// - String key: same as a string value
/// - Identifier key (`a:`, `undefined:`): always double-quoted
///   (identifier names cannot contain characters `JSON.stringify` would escape)
/// - Numeric key: see the module table for the mode split; in compat mode `String()`
///   (unlike value position's `JSON.stringify()`) has no `null` fallback, so non-finite quotes as `"Infinity"`
fn write_object_key<'a>(key: &PropertyKey<'a>, f: &mut JsonFormatter<'_, 'a>) {
    match key {
        PropertyKey::StringLiteral(lit) => write_string(lit, f),
        PropertyKey::StaticIdentifier(ident) => {
            write_quoted_str(f, b'"', ident.name.as_str());
        }
        PropertyKey::NumericLiteral(lit) => {
            let raw = lit.raw.as_ref().unwrap_or_else(|| unreachable!("parser always sets `raw`"));
            let raw = raw.as_str();
            if number_string_round_trips(raw) {
                write_quoted_str(f, b'"', raw);
            } else {
                write!(f, text(raw));
            }
        }
        _ => write!(f, FormatInvalidJson(key.span())),
    }
}

/// A template literal is JSON-printable only without substitutions:
/// `` `foo` `` → `"foo"` via `JSON.stringify(cooked)`
fn write_template<'a>(template: &TemplateLiteral<'a>, f: &mut JsonFormatter<'_, 'a>) {
    if !template.expressions.is_empty() || template.quasis.len() != 1 {
        write!(f, FormatInvalidJson(template.span));
        return;
    }
    let quasi = &template.quasis[0];
    // `cooked` is `None` for invalid escape sequences
    // (only possible in tagged templates, but stay defensive here).
    let Some(cooked) = &quasi.value.cooked else {
        write!(f, FormatInvalidJson(template.span));
        return;
    };
    let body = json_stringify_escape(cooked.as_str(), quasi.lone_surrogates);
    write_quoted_str(f, b'"', arena_cow_str(&body, f));
}

/// Escapes `content` the way `JSON.stringify` does for a string body
/// (the quotes themselves are not included): `"` and `\` get a backslash,
/// control characters use their short escapes (`\b\f\n\r\t`) or `\uXXXX`.
///
/// When `lone_surrogates` is set, `content` encodes each lone surrogate as
/// `\u{FFFD}XXXX` (and a literal U+FFFD as `\u{FFFD}fffd`).
/// The same scheme `oxc_codegen` decodes, `JSON.stringify` prints lone surrogates as `\uXXXX`.
fn json_stringify_escape(content: &str, lone_surrogates: bool) -> Cow<'_, str> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    if !lone_surrogates && !content.bytes().any(|b| matches!(b, b'"' | b'\\') || b < 0x20) {
        return Cow::Borrowed(content);
    }

    let mut out = String::with_capacity(content.len() + 8);
    // Start of the pending run of as-is characters, flushed in one `push_str`
    // before each escape (mirrors `oxc_formatter_core::spec::normalize_string`).
    let mut copy_start = 0;
    let mut chars = content.char_indices();
    while let Some((i, c)) = chars.next() {
        let short_escape = match c {
            '"' => Some("\\\""),
            '\\' => Some("\\\\"),
            '\u{8}' => Some("\\b"),
            '\u{c}' => Some("\\f"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\t' => Some("\\t"),
            _ => None,
        };
        if let Some(escaped) = short_escape {
            out.push_str(&content[copy_start..i]);
            out.push_str(escaped);
            copy_start = i + 1; // every short-escaped char is a single byte
        } else if (c as u32) < 0x20 {
            out.push_str(&content[copy_start..i]);
            // Always `\u00XX` — the guard caps the code point below 0x20.
            let code = c as u32;
            out.push_str("\\u00");
            out.push(HEX[(code >> 4) as usize] as char);
            out.push(HEX[(code & 0xF) as usize] as char);
            copy_start = i + 1;
        } else if c == '\u{FFFD}' && lone_surrogates {
            out.push_str(&content[copy_start..i]);
            // 4 lowercase-hex ASCII chars always follow the 3-byte escape marker;
            // `get` only guards against malformed input the parser never produces,
            // falling back (like the `fffd` self-escape) to a literal U+FFFD.
            let hex_start = i + 3;
            match content.get(hex_start..hex_start + 4) {
                Some(hex) if hex != "fffd" => {
                    out.push_str("\\u");
                    out.push_str(hex);
                }
                _ => out.push('\u{FFFD}'),
            }
            copy_start = (hex_start + 4).min(content.len());
            // Skip the 4 (single-byte) hex chars.
            for _ in 0..4 {
                chars.next();
            }
        }
    }
    out.push_str(&content[copy_start..]);
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::{json_stringify_escape, number_string_round_trips};

    #[test]
    fn key_round_trip() {
        // Quoted: survives `String(Number(raw))` unchanged
        for raw in ["0", "1", "0.1", "100", "1e+30", "1e-30"] {
            assert!(number_string_round_trips(raw), "{raw}");
        }
        // Unquoted: JS would print a different shape (or NaN)
        for raw in [
            "1.0",
            "1.00000",
            ".1",
            "1e2",
            "1.0e+2",
            "0x10",
            "1_2_3",
            "999999999999999999999999999999",
            "1e999",
        ] {
            assert!(!number_string_round_trips(raw), "{raw}");
        }
    }

    #[test]
    fn stringify_escape() {
        assert_eq!(json_stringify_escape("plain", false), "plain");
        assert_eq!(json_stringify_escape("a\"b\\c", false), "a\\\"b\\\\c");
        assert_eq!(json_stringify_escape("\u{8}\u{c}\n\r\t", false), "\\b\\f\\n\\r\\t");
        assert_eq!(json_stringify_escape("\0\u{1}\u{1f}", false), "\\u0000\\u0001\\u001f");
        // U+2028 / U+2029 are NOT escaped by `JSON.stringify`
        assert_eq!(json_stringify_escape("\u{2028}\u{2029}", false), "\u{2028}\u{2029}");
        // Lone surrogate `\u{FFFD}XXXX` encoding; `\u{FFFD}fffd` is a literal U+FFFD
        assert_eq!(json_stringify_escape("a\u{FFFD}d800b", true), "a\\ud800b");
        assert_eq!(json_stringify_escape("a\u{FFFD}fffdb", true), "a\u{FFFD}b");
    }
}

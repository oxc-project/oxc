use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::text},
    utils::string::{
        FormatLiteralStringToken, StringLiteralParentKind, is_identifier_name_patched,
    },
    write,
};

pub fn format_property_key<'a>(key: &AstNode<'a, PropertyKey<'a>>, f: &mut Formatter<'_, 'a>) {
    format_property_key_with_options(key, f, false);
}

/// Format a property key with options for preserving quotes.
/// `preserve_quotes` forces quotes to be preserved regardless of quoteProps setting.
pub fn format_property_key_with_options<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
    preserve_quotes: bool,
) {
    if let PropertyKey::StringLiteral(s) = key.as_ref() {
        // Use Expression kind if quotes should be preserved
        let kind = if preserve_quotes {
            StringLiteralParentKind::Expression
        } else {
            StringLiteralParentKind::Member
        };

        FormatLiteralStringToken::new(
            f.source_text().text_for(s.as_ref()),
            /* jsx */
            false,
            kind,
        )
        .fmt(f);
    } else if let PropertyKey::NumericLiteral(n) = key.as_ref() {
        // When quoteProps is "consistent" and quotes are needed, quote numeric keys
        // But only if they can be cleanly normalized without losing intent
        let raw = f.source_text().text_for(&n.span);
        let can_quote = can_quote_numeric_key(raw, n.value);

        if f.context().is_quote_needed() && can_quote {
            let quote_char = f.options().quote_style.as_byte() as char;
            let normalized = normalize_numeric_key(n.value);
            let quoted = format!("{quote_char}{normalized}{quote_char}");
            let allocated = f.context().allocator().alloc_str(&quoted);
            text(allocated).fmt(f);
        } else {
            write!(f, key);
        }
    } else {
        write!(f, key);
    }
}

pub fn write_member_name<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> usize {
    if let AstNodes::StringLiteral(string) = key.as_ast_nodes() {
        let format = FormatLiteralStringToken::new(
            f.source_text().text_for(string),
            false,
            StringLiteralParentKind::Member,
        )
        .clean_text(f);

        string.format_leading_comments(f);
        write!(f, format);
        string.format_trailing_comments(f);

        format.width()
    } else if let PropertyKey::NumericLiteral(n) = key.as_ref() {
        // When quoteProps is "consistent" and quotes are needed, quote numeric keys
        // But only if they can be cleanly normalized without losing intent
        let raw = f.source_text().text_for(&n.span);
        let can_quote = can_quote_numeric_key(raw, n.value);

        if f.context().is_quote_needed() && can_quote {
            let quote_char = f.options().quote_style.as_byte() as char;
            // Normalize the number (e.g., ".1" -> "0.1", "1." -> "1")
            let normalized = normalize_numeric_key(n.value);
            let quoted = format!("{quote_char}{normalized}{quote_char}");
            let allocated = f.context().allocator().alloc_str(&quoted);
            text(allocated).fmt(f);
            quoted.len()
        } else {
            write!(f, key);
            f.source_text().span_width(key.span())
        }
    } else {
        write!(f, key);

        f.source_text().span_width(key.span())
    }
}

/// Determine if the property key string literal should preserve its quotes
pub fn should_preserve_quote(key: &PropertyKey<'_>, f: &Formatter<'_, '_>) -> bool {
    matches!(&key, PropertyKey::StringLiteral(string) if {
        let quote_less_content = f.source_text().text_for(&string.span.shrink(1));
        !is_identifier_name_patched(quote_less_content)
    })
}

/// Check if a numeric literal source can be quoted without losing intent.
/// Only simple decimal forms like "1", "1.5", ".1", "1." can be quoted.
/// Complex forms like "1.0", "1E2", "0b10", "0o10", "0xf" should not be quoted.
fn can_quote_numeric_key(source: &str, value: f64) -> bool {
    let bytes = source.as_bytes();

    // Don't quote if it has exponent notation
    if source.contains('e') || source.contains('E') {
        return false;
    }

    // Don't quote binary, octal, or hex literals
    if bytes.len() >= 2 && bytes[0] == b'0' {
        let second = bytes[1];
        if second == b'b'
            || second == b'B'
            || second == b'o'
            || second == b'O'
            || second == b'x'
            || second == b'X'
        {
            return false;
        }
    }

    // Don't quote BigInt literals
    if source.ends_with('n') {
        return false;
    }

    // Don't quote if normalizing would change the numeric value
    // (e.g., 999999999999999999999 becomes 1e21 due to f64 limits)
    let normalized = normalize_numeric_key(value);

    // Compare the normalized value with what the source would normalize to
    // For simple cases like "1", "1.5", the normalized matches
    // For precision-losing cases like "999999999999999999999", normalized differs
    let source_normalized = normalize_source_numeric(source);
    if source_normalized != normalized {
        return false;
    }

    // Don't quote if there are digits after the decimal point followed by anything
    // (i.e., "1.0" has digits after decimal, so we keep it unquoted)
    if let Some(dot_pos) = source.find('.') {
        let after_dot = &source[dot_pos + 1..];

        if source.starts_with('.') {
            // ".1" style - can quote
            return true;
        }
        if after_dot.is_empty() {
            // "1." style - can quote
            return true;
        }
        // Check if any trailing zeros would be lost
        if after_dot.ends_with('0') {
            return false;
        }
    }

    true
}

/// Normalize a numeric value to its canonical string representation.
fn normalize_numeric_key(value: f64) -> String {
    // Use JavaScript-like number to string conversion
    // This handles cases like 0.1 -> "0.1", 1.0 -> "1", 1.5 -> "1.5"
    #[expect(clippy::cast_possible_truncation)]
    if value.fract() == 0.0 && value.abs() < 1e15 {
        // Integer-like value
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Normalize source text to a comparable numeric string.
/// Handles cases like ".1" -> "0.1", "1." -> "1"
fn normalize_source_numeric(source: &str) -> String {
    // Handle ".1" style (leading dot)
    if source.starts_with('.') {
        return format!("0{source}");
    }

    // Handle "1." style (trailing dot)
    if let Some(stripped) = source.strip_suffix('.') {
        return stripped.to_string();
    }

    // Otherwise return as-is (will be compared with f64 normalized)
    source.to_string()
}

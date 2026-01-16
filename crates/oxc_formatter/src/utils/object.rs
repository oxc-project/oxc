use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::number::ToJsString;

use crate::{
    Buffer, Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::token::number::{NumberFormatOptions, format_trimmed_number},
    formatter::{Formatter, prelude::text},
    utils::string::{
        FormatLiteralStringToken, StringLiteralParentKind, is_identifier_name_patched,
        is_simple_numeric_string,
    },
    write,
};

/// Format a property key, handling quoteProps: "consistent" mode.
pub fn format_property_key<'a>(key: &AstNode<'a, PropertyKey<'a>>, f: &mut Formatter<'_, 'a>) {
    let should_quote_for_consistency =
        f.options().quote_properties.is_consistent() && f.context().is_quote_needed();

    match key.as_ref() {
        PropertyKey::StringLiteral(s) => {
            // `"constructor"` property in the class should be kept quoted
            let kind = if matches!(key.parent, AstNodes::PropertyDefinition(_))
                && matches!(key.as_ref(), PropertyKey::StringLiteral(string) if string.value == "constructor")
            {
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
        }
        PropertyKey::StaticIdentifier(ident) if should_quote_for_consistency => {
            // Quote the identifier: foo → "foo" or 'foo'
            let quote = f.options().quote_style.as_str();
            let quoted = format!("{quote}{}{quote}", ident.name);
            text(f.context().allocator().alloc_str(&quoted)).fmt(f);
        }
        PropertyKey::NumericLiteral(num) if should_quote_for_consistency => {
            // Check if this is a simple number that can be quoted
            if let Some(value_str) = can_quote_numeric_literal(num, f) {
                let quote = f.options().quote_style.as_str();
                let quoted = format!("{quote}{value_str}{quote}");
                text(f.context().allocator().alloc_str(&quoted)).fmt(f);
            } else {
                // Complex number representation - don't quote
                write!(f, key);
            }
        }
        _ => {
            write!(f, key);
        }
    }
}

pub fn write_member_name<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> usize {
    // Check if we need to quote this key for quoteProps: "consistent"
    let should_quote_for_consistency =
        f.options().quote_properties.is_consistent() && f.context().is_quote_needed();

    match key.as_ref() {
        PropertyKey::StringLiteral(string) => {
            let format = FormatLiteralStringToken::new(
                f.source_text().text_for(string.as_ref()),
                false,
                StringLiteralParentKind::Member,
            )
            .clean_text(f);

            let string_node = key.as_ast_nodes();
            if let AstNodes::StringLiteral(s) = string_node {
                s.format_leading_comments(f);
                write!(f, format);
                s.format_trailing_comments(f);
            }

            format.width()
        }
        PropertyKey::StaticIdentifier(ident) if should_quote_for_consistency => {
            // Quote the identifier: foo → "foo" or 'foo'
            let quote = f.options().quote_style.as_str();
            let quoted = format!("{quote}{}{quote}", ident.name);
            let width = quoted.len();
            text(f.context().allocator().alloc_str(&quoted)).fmt(f);
            width
        }
        PropertyKey::NumericLiteral(num) if should_quote_for_consistency => {
            // Check if this is a simple number that can be quoted
            if let Some(value_str) = can_quote_numeric_literal(num, f) {
                let quote = f.options().quote_style.as_str();
                let quoted = format!("{quote}{value_str}{quote}");
                let width = quoted.len();
                text(f.context().allocator().alloc_str(&quoted)).fmt(f);
                width
            } else {
                // Complex number representation - don't quote
                write!(f, key);
                f.source_text().span_width(key.span())
            }
        }
        _ => {
            write!(f, key);
            f.source_text().span_width(key.span())
        }
    }
}

/// Determine if the property key string literal should preserve its quotes.
/// Returns true if the key requires quotes and cannot be safely unquoted.
///
/// A string key can be safely unquoted if:
/// - It's a valid JavaScript identifier (e.g., "foo" → foo), OR
/// - It's a simple numeric string that round-trips (e.g., "5" → 5, "1.5" → 1.5)
pub fn should_preserve_quote(key: &PropertyKey<'_>, f: &Formatter<'_, '_>) -> bool {
    matches!(&key, PropertyKey::StringLiteral(string) if {
        let quote_less_content = f.source_text().text_for(&string.span.shrink(1));
        !is_string_key_safe_to_unquote(quote_less_content)
    })
}

/// Check if a string key content can be safely unquoted.
/// Returns true if the key can become an identifier or a simple numeric literal.
fn is_string_key_safe_to_unquote(content: &str) -> bool {
    // Check if it's a valid identifier
    if is_identifier_name_patched(content) {
        return true;
    }

    // Check if it's a simple numeric string that round-trips correctly
    is_simple_numeric_string(content)
}

/// Check if a numeric literal can be quoted for quoteProps: "consistent".
///
/// A numeric literal can be quoted if its JavaScript string representation
/// (String(value)) matches the normalized printed form. This means simple
/// numbers like 1, 1.5, 0.1 can be quoted, but complex representations
/// like 1e2, 0b10, 0xf, 1.0 cannot.
///
/// Returns `Some(value_str)` if the number can be quoted, where `value_str`
/// is the string to use as the quoted key content.
fn can_quote_numeric_literal(num: &NumericLiteral<'_>, f: &Formatter<'_, '_>) -> Option<String> {
    let value = num.value;

    // NaN and Infinity cannot be represented as numeric literals
    if value.is_nan() || value.is_infinite() {
        return None;
    }

    // Get the JavaScript string representation using proper JS number-to-string conversion
    let value_str = value.to_js_string();

    // Get the normalized printed form using the same normalization as codegen
    let raw_text = f.source_text().text_for(num);
    let normalized =
        format_trimmed_number(raw_text, NumberFormatOptions::keep_one_trailing_decimal_zero());

    // They must match for the number to be quotable
    // For example:
    // - 1 → normalized "1", String(1) = "1" → match ✓
    // - 1.5 → normalized "1.5", String(1.5) = "1.5" → match ✓
    // - .1 → normalized "0.1", String(0.1) = "0.1" → match ✓
    // - 1. → normalized "1", String(1) = "1" → match ✓
    // - 1.0 → normalized "1.0", String(1) = "1" → no match ✗
    // - 1e2 → normalized "1e2", String(100) = "100" → no match ✗
    // - 0b10 → normalized "0b10", String(2) = "2" → no match ✗
    if value_str == normalized.as_ref() { Some(value_str) } else { None }
}

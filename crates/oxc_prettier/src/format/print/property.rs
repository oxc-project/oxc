use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name;

use crate::{array, dynamic_text, format::print::literal, ir::Doc, text, Format, Prettier};

pub fn is_property_key_has_quote(key: &PropertyKey<'_>) -> bool {
    matches!(key, PropertyKey::StringLiteral(literal) if is_string_prop_safe_to_unquote(literal.value.as_str()))
}

fn is_string_prop_safe_to_unquote(value: &str) -> bool {
    !is_identifier_name(value) && !is_simple_number(value)
}

// Matches “simple” numbers like `123` and `2.5` but not `1_000`, `1e+100` or `0b10`.
fn is_simple_number(str: &str) -> bool {
    let mut bytes = str.as_bytes().iter();
    let mut has_dot = false;
    bytes.next().is_some_and(u8::is_ascii_digit)
        && bytes.all(|c| {
            if c == &b'.' {
                if has_dot {
                    return false;
                }
                has_dot = true;
                return true;
            }
            c.is_ascii_digit()
        })
}

pub enum PropertyKeyLike<'a, 'b> {
    ImportAttributeKey(&'b ImportAttributeKey<'a>),
    PropertyKey(&'b PropertyKey<'a>),
}

pub fn print_property_key<'a>(
    p: &mut Prettier<'a>,
    property_key: &PropertyKeyLike<'a, '_>,
    is_computed: bool,
    has_quote_props: bool,
) -> Doc<'a> {
    if let PropertyKeyLike::PropertyKey(property_key) = property_key {
        if is_computed {
            let key_doc = match property_key {
                PropertyKey::StaticIdentifier(ident) => ident.format(p),
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                match_expression!(PropertyKey) => property_key.to_expression().format(p),
            };
            return array!(p, [text!("["), key_doc, text!("]")]);
        }
    }

    match property_key {
        PropertyKeyLike::ImportAttributeKey(import_attribute_key) => match import_attribute_key {
            // TODO: Apply the same rule as `PropertyKey`
            ImportAttributeKey::Identifier(ident) => ident.format(p),
            ImportAttributeKey::StringLiteral(literal) => literal.format(p),
        },
        PropertyKeyLike::PropertyKey(property_key) => {
            let need_quote = p.options.quote_props.consistent() && has_quote_props;

            match property_key {
                PropertyKey::StaticIdentifier(ident) => {
                    if need_quote {
                        literal::print_string_from_not_quoted_raw_text(
                            p,
                            &ident.name,
                            p.options.single_quote,
                        )
                    } else {
                        ident.format(p)
                    }
                }
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                PropertyKey::StringLiteral(literal) => {
                    // This does not pass quotes/objects.js
                    // because prettier uses the function `isEs5IdentifierName` based on unicode version 3,
                    // but `is_identifier_name` uses the latest unicode version.
                    if is_identifier_name(literal.value.as_str())
                        && (p.options.quote_props.as_needed()
                            || (p.options.quote_props.consistent()/* && !needsQuoteProps.get(parent) */))
                    {
                        dynamic_text!(p, literal.value.as_str())
                    } else {
                        literal::print_string_from_not_quoted_raw_text(
                            p,
                            literal.value.as_str(),
                            p.options.single_quote,
                        )
                    }
                }
                PropertyKey::NumericLiteral(literal) => {
                    if need_quote {
                        literal::print_string_from_not_quoted_raw_text(
                            p,
                            &literal.raw_str(),
                            p.options.single_quote,
                        )
                    } else {
                        literal.format(p)
                    }
                }
                PropertyKey::Identifier(ident) => {
                    let ident_doc = ident.format(p);
                    array!(p, [text!("["), ident_doc, text!("]")])
                }
                match_expression!(PropertyKey) => property_key.to_expression().format(p),
            }
        }
    }
}

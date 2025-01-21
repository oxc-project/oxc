use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};
use oxc_syntax::identifier::is_identifier_name;

use crate::{array, dynamic_text, format::print::literal, ir::Doc, text, Format, Prettier};

pub enum PropertyKeyLike<'a, 'b> {
    ImportAttributeKey(&'b ImportAttributeKey<'a>),
    PropertyKey(&'b PropertyKey<'a>),
}

pub fn print_property_key<'a>(
    p: &mut Prettier<'a>,
    property_key: &PropertyKeyLike<'a, '_>,
    is_computed: bool,
) -> Doc<'a> {
    if let PropertyKeyLike::PropertyKey(property_key) = property_key {
        if is_computed {
            return array!(p, [text!("["), property_key.format(p), text!("]")]);
        }
    }

    // PERF: Cache this result by key-holder to avoid re-calculation by each property
    let needs_quote = p.options.quote_props.consistent()
        && match p.parent_kind() {
            AstKind::ObjectExpression(oe) => oe.properties.iter().any(|opk| match opk {
                ObjectPropertyKind::ObjectProperty(p) => {
                    !p.computed && is_property_key_has_quote(&p.key)
                }
                ObjectPropertyKind::SpreadProperty(_) => false,
            }),
            AstKind::ClassBody(cb) => cb.body.iter().any(|ce| match ce {
                ClassElement::PropertyDefinition(d) => {
                    !d.computed && is_property_key_has_quote(&d.key)
                }
                _ => false,
            }),
            _ => false,
        };

    match property_key {
        PropertyKeyLike::ImportAttributeKey(import_attribute_key) => match import_attribute_key {
            ImportAttributeKey::Identifier(ident) => {
                if needs_quote {
                    literal::print_string_from_not_quoted_raw_text(
                        p,
                        &ident.name,
                        p.options.single_quote,
                    )
                } else {
                    ident.format(p)
                }
            }
            ImportAttributeKey::StringLiteral(literal) => {
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
        },
        PropertyKeyLike::PropertyKey(property_key) => {
            match property_key {
                PropertyKey::StaticIdentifier(ident) => {
                    if needs_quote {
                        literal::print_string_from_not_quoted_raw_text(
                            p,
                            &ident.name,
                            p.options.single_quote,
                        )
                    } else {
                        ident.format(p)
                    }
                }
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
                    if needs_quote {
                        literal::print_string_from_not_quoted_raw_text(
                            p,
                            &literal.raw_str(),
                            p.options.single_quote,
                        )
                    } else {
                        literal.format(p)
                    }
                }
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                PropertyKey::Identifier(ident) => ident.format(p),
                match_expression!(PropertyKey) => property_key.to_expression().format(p),
            }
        }
    }
}

fn is_property_key_has_quote(key: &PropertyKey<'_>) -> bool {
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

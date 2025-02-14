use oxc_ast::ast::*;

use crate::is_global_reference::IsGlobalReference;

/// `ToPrimitive`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toprimitive>
pub trait ToPrimitive<'a> {
    fn to_primitive_returns_string(&self) -> Option<bool>;
    fn to_primitive_returns_symbol(
        &self,
        is_global_reference: &impl IsGlobalReference,
    ) -> Option<bool>;
    fn to_primitive_returns_symbol_or_bigint(
        &self,
        is_global_reference: &impl IsGlobalReference,
    ) -> Option<bool>;
}

impl ToPrimitive<'_> for Expression<'_> {
    fn to_primitive_returns_string(&self) -> Option<bool> {
        match self {
            Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_) => Some(true),
            // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
            // ToPrimitive for an object returns `"[object Object]"`
            Expression::ObjectExpression(obj) => {
                if maybe_object_with_to_primitive_related_properties_overridden(obj) {
                    None
                } else {
                    Some(true)
                }
            }
            _ => None,
        }
    }

    fn to_primitive_returns_symbol(
        &self,
        is_global_reference: &impl IsGlobalReference,
    ) -> Option<bool> {
        match self {
            Expression::Identifier(ident) => {
                if matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                    && is_global_reference.is_global_reference(ident) == Some(true)
                {
                    Some(false)
                } else {
                    None
                }
            }
            Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_) => Some(false),
            // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
            // ToPrimitive for an object returns `"[object Object]"`
            Expression::ObjectExpression(obj) => {
                if maybe_object_with_to_primitive_related_properties_overridden(obj) {
                    None
                } else {
                    Some(false)
                }
            }
            _ => None,
        }
    }

    fn to_primitive_returns_symbol_or_bigint(
        &self,
        is_global_reference: &impl IsGlobalReference,
    ) -> Option<bool> {
        match self {
            Expression::Identifier(ident) => {
                if matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                    && is_global_reference.is_global_reference(ident) == Some(true)
                {
                    Some(false)
                } else {
                    None
                }
            }
            Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_) => Some(false),
            // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
            // ToPrimitive for an object returns `"[object Object]"`
            Expression::ObjectExpression(obj) => {
                if maybe_object_with_to_primitive_related_properties_overridden(obj) {
                    None
                } else {
                    Some(false)
                }
            }
            _ => None,
        }
    }
}

pub(crate) fn maybe_object_with_to_primitive_related_properties_overridden(
    obj: &ObjectExpression<'_>,
) -> bool {
    obj.properties.iter().any(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
            PropertyKey::StaticIdentifier(id) => {
                matches!(id.name.as_str(), "toString" | "valueOf")
            }
            PropertyKey::PrivateIdentifier(_) => false,
            PropertyKey::StringLiteral(str) => {
                matches!(str.value.as_str(), "toString" | "valueOf")
            }
            PropertyKey::TemplateLiteral(temp) => {
                !temp.is_no_substitution_template()
                    || temp
                        .quasi()
                        .is_some_and(|val| matches!(val.as_str(), "toString" | "valueOf"))
            }
            _ => true,
        },
        ObjectPropertyKind::SpreadProperty(e) => match &e.argument {
            Expression::ObjectExpression(obj) => {
                maybe_object_with_to_primitive_related_properties_overridden(obj)
            }
            Expression::ArrayExpression(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_) => false,
            _ => true,
        },
    })
}

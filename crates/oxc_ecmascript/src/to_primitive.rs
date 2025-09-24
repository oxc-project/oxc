use oxc_ast::ast::*;

use crate::{
    GlobalContext,
    constant_evaluation::{DetermineValueType, ValueType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToPrimitiveResult {
    Undefined,
    Null,
    Number,
    BigInt,
    String,
    Boolean,
    Symbol,
    Undetermined,
}

impl ToPrimitiveResult {
    pub fn is_string(self) -> Option<bool> {
        match self {
            Self::Undetermined => None,
            Self::String => Some(true),
            _ => Some(false),
        }
    }

    pub fn is_symbol(self) -> Option<bool> {
        match self {
            Self::Undetermined => None,
            Self::Symbol => Some(true),
            _ => Some(false),
        }
    }

    pub fn is_symbol_or_bigint(self) -> Option<bool> {
        match self {
            Self::Undetermined => None,
            Self::BigInt | Self::Symbol => Some(true),
            _ => Some(false),
        }
    }
}

/// `ToPrimitive`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toprimitive>
pub trait ToPrimitive<'a> {
    fn to_primitive(&self, ctx: &impl GlobalContext<'a>) -> ToPrimitiveResult;
}

impl<'a> ToPrimitive<'a> for Expression<'a> {
    fn to_primitive(&self, ctx: &impl GlobalContext<'a>) -> ToPrimitiveResult {
        match self.value_type(ctx) {
            ValueType::Undefined => ToPrimitiveResult::Undefined,
            ValueType::Null => ToPrimitiveResult::Null,
            ValueType::Number => ToPrimitiveResult::Number,
            ValueType::BigInt => ToPrimitiveResult::BigInt,
            ValueType::String => ToPrimitiveResult::String,
            ValueType::Boolean => ToPrimitiveResult::Boolean,
            ValueType::Object | ValueType::Undetermined => {
                match self {
                    Expression::RegExpLiteral(_) | Expression::ArrayExpression(_) => {
                        ToPrimitiveResult::String
                    }
                    // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
                    // ToPrimitive for an object returns `"[object Object]"`
                    Expression::ObjectExpression(obj) => {
                        if maybe_object_with_to_primitive_related_properties_overridden(obj) {
                            ToPrimitiveResult::Undetermined
                        } else {
                            ToPrimitiveResult::String
                        }
                    }
                    _ => ToPrimitiveResult::Undetermined,
                }
            }
        }
    }
}

pub fn maybe_object_with_to_primitive_related_properties_overridden(
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
            PropertyKey::TemplateLiteral(temp) => temp
                .single_quasi()
                .is_some_and(|val| matches!(val.as_str(), "toString" | "valueOf")),
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

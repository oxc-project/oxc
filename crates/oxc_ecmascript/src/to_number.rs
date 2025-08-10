use oxc_ast::ast::*;

use crate::{
    GlobalContext, to_primitive::maybe_object_with_to_primitive_related_properties_overridden,
};

/// `ToNumber`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber>
pub trait ToNumber<'a> {
    fn to_number(&self, ctx: &impl GlobalContext<'a>) -> Option<f64>;
}

impl<'a> ToNumber<'a> for Expression<'a> {
    fn to_number(&self, ctx: &impl GlobalContext<'a>) -> Option<f64> {
        match self {
            Expression::NumericLiteral(number_literal) => Some(number_literal.value),
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }
            Expression::NullLiteral(_) => Some(0.0),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "Infinity" if ctx.is_global_reference(ident) => Some(f64::INFINITY),
                "NaN" | "undefined" if ctx.is_global_reference(ident) => Some(f64::NAN),
                _ => None,
            },
            Expression::StringLiteral(lit) => {
                use crate::StringToNumber;
                Some(lit.value.as_str().string_to_number())
            }
            Expression::UnaryExpression(unary) if unary.operator.is_not() => {
                let number = unary.argument.to_number(ctx)?;
                Some(if number == 0.0 { 1.0 } else { 0.0 })
            }
            Expression::ObjectExpression(obj) => {
                // If `toString` / `valueOf` / `Symbol.toPrimitive` is not overridden,
                // (assuming that those methods in Object.prototype are not modified)
                // `ToPrimitive` returns `"[object Object]"`
                if maybe_object_with_to_primitive_related_properties_overridden(obj) {
                    None
                } else {
                    Some(f64::NAN)
                }
            }
            // `ToPrimitive` for RegExp object returns `"/regexp/"`
            Expression::RegExpLiteral(_) => Some(f64::NAN),
            Expression::ArrayExpression(arr) => {
                // If the array is empty, `ToPrimitive` returns `""`
                if arr.elements.is_empty() {
                    return Some(0.0);
                }
                if arr.elements.len() == 1 {
                    let first_element = arr.elements.first().unwrap();
                    return match first_element {
                        ArrayExpressionElement::SpreadElement(_) => None,
                        // `ToPrimitive` returns `""` for `[,]`
                        ArrayExpressionElement::Elision(_) => Some(0.0),
                        match_expression!(ArrayExpressionElement) => {
                            first_element.to_expression().to_number(ctx)
                        }
                    };
                }

                let non_spread_element_count = arr
                    .elements
                    .iter()
                    .filter(|e| !matches!(e, ArrayExpressionElement::SpreadElement(_)))
                    .count();
                // If the array has at least 2 elements, `ToPrimitive` returns a string containing
                // `,` which is not included in `StringNumericLiteral`
                // So `ToNumber` returns `NaN`
                if non_spread_element_count >= 2 { Some(f64::NAN) } else { None }
            }
            _ => None,
        }
    }
}

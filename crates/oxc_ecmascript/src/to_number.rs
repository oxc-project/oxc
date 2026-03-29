use oxc_ast::ast::*;

use crate::{
    GlobalContext, StringToNumber, ToJsString,
    to_primitive::maybe_object_with_to_primitive_related_properties_overridden,
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
            Expression::StringLiteral(lit) => Some(lit.value.as_str().string_to_number()),
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
                // ToNumber for arrays:
                // 1. ToPrimitive(array, hint Number) -> tries valueOf, then toString
                // 2. Array.toString() -> Array.join(",")
                // 3. ToNumber(resultString)

                // Fast path: if array has at least 2 non-spread elements,
                // the result will contain "," which converts to NaN
                if arr
                    .elements
                    .iter()
                    .filter(|e| !matches!(e, ArrayExpressionElement::SpreadElement(_)))
                    .take(2)
                    .count()
                    >= 2
                {
                    return Some(f64::NAN);
                }

                let array_string = arr.to_js_string(ctx)?;
                Some(array_string.as_ref().string_to_number())
            }
            _ => None,
        }
    }
}

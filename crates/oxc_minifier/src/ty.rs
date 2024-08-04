#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

/// JavaScript Language Type
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    BigInt,
    Boolean,
    Null,
    Number,
    Object,
    Str,
    Void,
    Undetermined,
}

impl<'a> From<&Expression<'a>> for Ty {
    fn from(expr: &Expression<'a>) -> Self {
        // TODO: complete this
        match expr {
            Expression::BigIntLiteral(_) => Self::BigInt,
            Expression::BooleanLiteral(_) => Self::Boolean,
            Expression::NullLiteral(_) => Self::Null,
            Expression::NumericLiteral(_) => Self::Number,
            Expression::StringLiteral(_) => Self::Str,
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_) => Self::Object,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Self::Void,
                "NaN" | "Infinity" => Self::Number,
                _ => Self::Undetermined,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Void => Self::Void,
                UnaryOperator::UnaryNegation => {
                    let argument_ty = Self::from(&unary_expr.argument);
                    if argument_ty == Self::BigInt {
                        return Self::BigInt;
                    }
                    Self::Number
                }
                UnaryOperator::UnaryPlus => Self::Number,
                UnaryOperator::LogicalNot => Self::Boolean,
                UnaryOperator::Typeof => Self::Str,
                _ => Self::Undetermined,
            },
            Expression::BinaryExpression(binary_expr) => match binary_expr.operator {
                BinaryOperator::Addition => {
                    let left_ty = Self::from(&binary_expr.left);
                    let right_ty = Self::from(&binary_expr.right);

                    if left_ty == Self::Str || right_ty == Self::Str {
                        return Self::Str;
                    }

                    // There are some pretty weird cases for object types:
                    //   {} + [] === "0"
                    //   [] + {} === "[object Object]"
                    if left_ty == Self::Object || right_ty == Self::Object {
                        return Self::Undetermined;
                    }

                    Self::Undetermined
                }
                _ => Self::Undetermined,
            },
            _ => Self::Undetermined,
        }
    }
}

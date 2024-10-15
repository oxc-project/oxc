use oxc_ast::ast::Expression;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

/// JavaScript Language Type
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Undefined, // a.k.a `Void` in closure compiler
    Null,
    Number,
    BigInt,
    String,
    Boolean,
    Object,
    Undetermined,
}

impl ValueType {
    pub fn is_string(self) -> bool {
        matches!(self, Self::String)
    }

    pub fn is_number(self) -> bool {
        matches!(self, Self::Number)
    }
}

/// `get_known_value_type`
///
/// Evaluate  and attempt to determine which primitive value type it could resolve to.
/// Without proper type information some assumptions had to be made for operations that could
/// result in a BigInt or a Number. If there is not enough information available to determine one
/// or the other then we assume Number in order to maintain historical behavior of the compiler and
/// avoid breaking projects that relied on this behavior.
impl<'a> From<&Expression<'a>> for ValueType {
    fn from(expr: &Expression<'a>) -> Self {
        // TODO: complete this
        match expr {
            Expression::BigIntLiteral(_) => Self::BigInt,
            Expression::BooleanLiteral(_) => Self::Boolean,
            Expression::NullLiteral(_) => Self::Null,
            Expression::NumericLiteral(_) => Self::Number,
            Expression::StringLiteral(_) => Self::String,
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_) => Self::Object,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Self::Undefined,
                "NaN" | "Infinity" => Self::Number,
                _ => Self::Undetermined,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Void => Self::Undefined,
                UnaryOperator::UnaryNegation => {
                    let argument_ty = Self::from(&unary_expr.argument);
                    if argument_ty == Self::BigInt {
                        return Self::BigInt;
                    }
                    Self::Number
                }
                UnaryOperator::UnaryPlus => Self::Number,
                UnaryOperator::LogicalNot => Self::Boolean,
                UnaryOperator::Typeof => Self::String,
                _ => Self::Undetermined,
            },
            Expression::BinaryExpression(binary_expr) => match binary_expr.operator {
                BinaryOperator::Addition => {
                    let left_ty = Self::from(&binary_expr.left);
                    let right_ty = Self::from(&binary_expr.right);
                    if left_ty == Self::String || right_ty == Self::String {
                        return Self::String;
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
            Expression::SequenceExpression(e) => {
                e.expressions.last().map_or(ValueType::Undetermined, Self::from)
            }
            _ => Self::Undetermined,
        }
    }
}

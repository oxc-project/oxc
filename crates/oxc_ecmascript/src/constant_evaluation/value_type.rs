use oxc_ast::ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, ConditionalExpression, Expression,
    LogicalExpression,
};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::is_global_reference::IsGlobalReference;

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
    pub fn is_undefined(self) -> bool {
        self == Self::Undefined
    }

    pub fn is_null(self) -> bool {
        self == Self::Null
    }

    pub fn is_string(self) -> bool {
        self == Self::String
    }

    pub fn is_number(self) -> bool {
        self == Self::Number
    }

    pub fn is_bigint(self) -> bool {
        self == Self::BigInt
    }

    pub fn is_boolean(self) -> bool {
        self == Self::Boolean
    }

    pub fn is_object(self) -> bool {
        self == Self::Object
    }

    pub fn is_undetermined(self) -> bool {
        self == Self::Undetermined
    }
}

/// Based on `get_known_value_type` in closure compiler
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/NodeUtil.java#L1517>
///
/// Evaluate the expression and attempt to determine which ValueType it could resolve to.
/// This function ignores the cases that throws an error, e.g. `foo * 0` can throw an error when `foo` is a bigint.
/// To detect those cases, use [`crate::side_effects::MayHaveSideEffects::expression_may_have_side_effects`].
pub trait DetermineValueType: IsGlobalReference {
    fn expression_value_type(&self, expr: &Expression<'_>) -> ValueType {
        match expr {
            Expression::BigIntLiteral(_) => ValueType::BigInt,
            Expression::BooleanLiteral(_) | Expression::PrivateInExpression(_) => {
                ValueType::Boolean
            }
            Expression::NullLiteral(_) => ValueType::Null,
            Expression::NumericLiteral(_) => ValueType::Number,
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => ValueType::String,
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_) => ValueType::Object,
            Expression::MetaProperty(meta_prop) => {
                match (meta_prop.meta.name.as_str(), meta_prop.property.name.as_str()) {
                    ("import", "meta") => ValueType::Object,
                    _ => ValueType::Undetermined,
                }
            }
            Expression::Identifier(ident) => {
                if self.is_global_reference(ident) == Some(true) {
                    match ident.name.as_str() {
                        "undefined" => ValueType::Undefined,
                        "NaN" | "Infinity" => ValueType::Number,
                        _ => ValueType::Undetermined,
                    }
                } else {
                    ValueType::Undetermined
                }
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Void => ValueType::Undefined,
                UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                    let argument_ty = self.expression_value_type(&unary_expr.argument);
                    match argument_ty {
                        ValueType::BigInt => ValueType::BigInt,
                        // non-object values other than BigInt are converted to number by `ToNumber`
                        ValueType::Number
                        | ValueType::Boolean
                        | ValueType::String
                        | ValueType::Null
                        | ValueType::Undefined => ValueType::Number,
                        ValueType::Undetermined | ValueType::Object => ValueType::Undetermined,
                    }
                }
                UnaryOperator::UnaryPlus => ValueType::Number,
                UnaryOperator::LogicalNot | UnaryOperator::Delete => ValueType::Boolean,
                UnaryOperator::Typeof => ValueType::String,
            },
            Expression::BinaryExpression(e) => self.binary_expression_value_type(e),
            Expression::SequenceExpression(e) => e
                .expressions
                .last()
                .map_or(ValueType::Undetermined, |e| self.expression_value_type(e)),
            Expression::AssignmentExpression(e) => self.assignment_expression_value_type(e),
            Expression::ConditionalExpression(e) => self.conditional_expression_value_type(e),
            Expression::LogicalExpression(e) => self.logical_expression_value_type(e),
            Expression::ParenthesizedExpression(e) => self.expression_value_type(&e.expression),
            _ => ValueType::Undetermined,
        }
    }

    fn binary_expression_value_type(&self, e: &BinaryExpression<'_>) -> ValueType {
        match e.operator {
            BinaryOperator::Addition => {
                let left = self.expression_value_type(&e.left);
                let right = self.expression_value_type(&e.right);
                if left == ValueType::Boolean
                    && matches!(right, ValueType::Undefined | ValueType::Null | ValueType::Number)
                {
                    return ValueType::Number;
                }
                if left == ValueType::String || right == ValueType::String {
                    return ValueType::String;
                }
                // There are some pretty weird cases for object types:
                //   {} + [] === "0"
                //   [] + {} === "[object Object]"
                if left == ValueType::Object || right == ValueType::Object {
                    return ValueType::Undetermined;
                }
                ValueType::Undetermined
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::BitwiseOR
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::Exponential => {
                let left = self.expression_value_type(&e.left);
                let right = self.expression_value_type(&e.right);
                if left.is_bigint() || right.is_bigint() {
                    ValueType::BigInt
                } else if !(left.is_object() || left.is_undetermined())
                    || !(right.is_object() || right.is_undetermined())
                {
                    // non-object values other than BigInt are converted to number by `ToNumber`
                    // if either operand is a number, the result is always a number
                    // because if the other operand is a bigint, an error is thrown
                    ValueType::Number
                } else {
                    ValueType::Undetermined
                }
            }
            BinaryOperator::ShiftRightZeroFill => ValueType::Number,
            BinaryOperator::Instanceof
            | BinaryOperator::In
            | BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => ValueType::Boolean,
        }
    }

    fn assignment_expression_value_type(&self, e: &AssignmentExpression<'_>) -> ValueType {
        match e.operator {
            AssignmentOperator::Assign => self.expression_value_type(&e.right),
            AssignmentOperator::Addition => {
                let right = self.expression_value_type(&e.right);
                if right.is_string() {
                    ValueType::String
                } else {
                    ValueType::Undetermined
                }
            }
            AssignmentOperator::Subtraction
            | AssignmentOperator::Multiplication
            | AssignmentOperator::Division
            | AssignmentOperator::Remainder
            | AssignmentOperator::ShiftLeft
            | AssignmentOperator::BitwiseOR
            | AssignmentOperator::ShiftRight
            | AssignmentOperator::BitwiseXOR
            | AssignmentOperator::BitwiseAnd
            | AssignmentOperator::Exponential => {
                let right = self.expression_value_type(&e.right);
                if right.is_bigint() {
                    ValueType::BigInt
                } else if !(right.is_object() || right.is_undetermined()) {
                    ValueType::Number
                } else {
                    ValueType::Undetermined
                }
            }
            AssignmentOperator::ShiftRightZeroFill => ValueType::Number,
            AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalNullish => ValueType::Undetermined,
        }
    }

    fn conditional_expression_value_type(&self, e: &ConditionalExpression<'_>) -> ValueType {
        let left = self.expression_value_type(&e.consequent);
        if left.is_undetermined() {
            return ValueType::Undetermined;
        }
        let right = self.expression_value_type(&e.alternate);
        if left == right {
            return left;
        }
        ValueType::Undetermined
    }

    fn logical_expression_value_type(&self, e: &LogicalExpression<'_>) -> ValueType {
        let left = self.expression_value_type(&e.left);
        if !left.is_boolean() {
            return ValueType::Undetermined;
        }
        let right = self.expression_value_type(&e.right);
        if left == right {
            return left;
        }
        ValueType::Undetermined
    }
}

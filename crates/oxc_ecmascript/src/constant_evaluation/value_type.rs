use oxc_ast::ast::*;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    is_global_reference::IsGlobalReference, to_numeric::ToNumeric, to_primitive::ToPrimitive,
};

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
/// To detect those cases, use [`crate::side_effects::MayHaveSideEffects`].
pub trait DetermineValueType<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType;
}

impl<'a> DetermineValueType<'a> for Expression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        match self {
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
                if is_global_reference.is_global_reference(ident) == Some(true) {
                    match ident.name.as_str() {
                        "undefined" => ValueType::Undefined,
                        "NaN" | "Infinity" => ValueType::Number,
                        _ => ValueType::Undetermined,
                    }
                } else {
                    ValueType::Undetermined
                }
            }
            Expression::UnaryExpression(e) => e.value_type(is_global_reference),
            Expression::BinaryExpression(e) => e.value_type(is_global_reference),
            Expression::SequenceExpression(e) => e
                .expressions
                .last()
                .map_or(ValueType::Undetermined, |e| e.value_type(is_global_reference)),
            Expression::AssignmentExpression(e) => e.value_type(is_global_reference),
            Expression::ConditionalExpression(e) => e.value_type(is_global_reference),
            Expression::LogicalExpression(e) => e.value_type(is_global_reference),
            Expression::ParenthesizedExpression(e) => e.expression.value_type(is_global_reference),
            Expression::StaticMemberExpression(e) => e.value_type(is_global_reference),
            Expression::NewExpression(e) => e.value_type(is_global_reference),
            _ => ValueType::Undetermined,
        }
    }
}

impl<'a> DetermineValueType<'a> for BinaryExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        match self.operator {
            BinaryOperator::Addition => {
                let left = self.left.to_primitive(is_global_reference);
                let right = self.right.to_primitive(is_global_reference);
                if left.is_string() == Some(true) || right.is_string() == Some(true) {
                    return ValueType::String;
                }
                let left_to_numeric_type = left.to_numeric(is_global_reference);
                let right_to_numeric_type = right.to_numeric(is_global_reference);
                // we need to check both operands because the other operand might be undetermined and maybe a string
                if left_to_numeric_type.is_number() && right_to_numeric_type.is_number() {
                    return ValueType::Number;
                }
                if left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint() {
                    return ValueType::BigInt;
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
                let left_to_numeric_type = self.left.to_numeric(is_global_reference);
                let right_to_numeric_type = self.right.to_numeric(is_global_reference);
                // if either operand is a number, the result is always a number
                // because if the other operand is a bigint, an error is thrown
                if left_to_numeric_type.is_number() || right_to_numeric_type.is_number() {
                    ValueType::Number
                } else if left_to_numeric_type.is_bigint() || right_to_numeric_type.is_bigint() {
                    ValueType::BigInt
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
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterEqualThan => ValueType::Boolean,
        }
    }
}

impl<'a> DetermineValueType<'a> for UnaryExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        match self.operator {
            UnaryOperator::Void => ValueType::Undefined,
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                let argument_ty = self.argument.value_type(is_global_reference);
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
        }
    }
}

impl<'a> DetermineValueType<'a> for AssignmentExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        match self.operator {
            AssignmentOperator::Assign => self.right.value_type(is_global_reference),
            AssignmentOperator::Addition => {
                let right = self.right.value_type(is_global_reference);
                if right.is_string() { ValueType::String } else { ValueType::Undetermined }
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
                let right = self.right.value_type(is_global_reference);
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
}

impl<'a> DetermineValueType<'a> for ConditionalExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        let left = self.consequent.value_type(is_global_reference);
        if left.is_undetermined() {
            return ValueType::Undetermined;
        }
        let right = self.alternate.value_type(is_global_reference);
        if left == right {
            return left;
        }
        ValueType::Undetermined
    }
}

impl<'a> DetermineValueType<'a> for LogicalExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        match self.operator {
            LogicalOperator::And | LogicalOperator::Or => {
                let left = self.left.value_type(is_global_reference);
                if left.is_undetermined() {
                    return ValueType::Undetermined;
                }
                let right = self.right.value_type(is_global_reference);
                if left == right {
                    return left;
                }
                ValueType::Undetermined
            }
            LogicalOperator::Coalesce => {
                let left = self.left.value_type(is_global_reference);
                match left {
                    ValueType::Undefined | ValueType::Null => {
                        self.right.value_type(is_global_reference)
                    }
                    ValueType::Undetermined => ValueType::Undetermined,
                    _ => left,
                }
            }
        }
    }
}

impl<'a> DetermineValueType<'a> for StaticMemberExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        if matches!(self.property.name.as_str(), "POSITIVE_INFINITY" | "NEGATIVE_INFINITY") {
            if let Some(ident) = self.object.get_identifier_reference() {
                if ident.name.as_str() == "Number"
                    && is_global_reference.is_global_reference(ident) == Some(true)
                {
                    return ValueType::Number;
                }
            }
        }
        ValueType::Undetermined
    }
}

impl<'a> DetermineValueType<'a> for NewExpression<'a> {
    fn value_type(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ValueType {
        if let Some(ident) = self.callee.get_identifier_reference() {
            if ident.name.as_str() == "Date"
                && is_global_reference.is_global_reference(ident) == Some(true)
            {
                return ValueType::Object;
            }
        }
        ValueType::Undetermined
    }
}

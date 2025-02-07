use oxc_ast::ast::{BinaryExpression, ConditionalExpression, Expression, LogicalExpression};
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

impl<'a> From<&Expression<'a>> for ValueType {
    /// Based on `get_known_value_type` in closure compiler
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/NodeUtil.java#L1517>
    ///
    /// Evaluate the expression and attempt to determine which ValueType it could resolve to.
    /// This function ignores the cases that throws an error, e.g. `foo * 0` can throw an error when `foo` is a bigint.
    /// To detect those cases, use [`crate::side_effects::MayHaveSideEffects::expression_may_have_side_effects`].
    fn from(expr: &Expression<'a>) -> Self {
        // TODO: complete this
        match expr {
            Expression::BigIntLiteral(_) => Self::BigInt,
            Expression::BooleanLiteral(_) => Self::Boolean,
            Expression::NullLiteral(_) => Self::Null,
            Expression::NumericLiteral(_) => Self::Number,
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => Self::String,
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
                    match argument_ty {
                        Self::BigInt => Self::BigInt,
                        // non-object values other than BigInt are converted to number by `ToNumber`
                        Self::Number
                        | Self::Boolean
                        | Self::String
                        | Self::Null
                        | Self::Undefined => Self::Number,
                        Self::Undetermined | Self::Object => Self::Undetermined,
                    }
                }
                UnaryOperator::UnaryPlus => Self::Number,
                UnaryOperator::LogicalNot | UnaryOperator::Delete => Self::Boolean,
                UnaryOperator::Typeof => Self::String,
                UnaryOperator::BitwiseNot => Self::Undetermined,
            },
            Expression::BinaryExpression(e) => Self::from(&**e),
            Expression::SequenceExpression(e) => {
                e.expressions.last().map_or(ValueType::Undetermined, Self::from)
            }
            Expression::AssignmentExpression(e) => Self::from(&e.right),
            Expression::ConditionalExpression(e) => Self::from(&**e),
            Expression::LogicalExpression(e) => Self::from(&**e),
            _ => Self::Undetermined,
        }
    }
}

impl<'a> From<&BinaryExpression<'a>> for ValueType {
    fn from(e: &BinaryExpression<'a>) -> Self {
        match e.operator {
            BinaryOperator::Addition => {
                let left = Self::from(&e.left);
                let right = Self::from(&e.right);
                if left == Self::Boolean
                    && matches!(right, Self::Undefined | Self::Null | Self::Number)
                {
                    return Self::Number;
                }
                if left == Self::String || right == Self::String {
                    return Self::String;
                }
                // There are some pretty weird cases for object types:
                //   {} + [] === "0"
                //   [] + {} === "[object Object]"
                if left == Self::Object || right == Self::Object {
                    return Self::Undetermined;
                }
                Self::Undetermined
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
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftRightZeroFill => Self::Number,
            BinaryOperator::Instanceof
            | BinaryOperator::In
            | BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => Self::Boolean,
        }
    }
}

impl<'a> From<&ConditionalExpression<'a>> for ValueType {
    fn from(e: &ConditionalExpression<'a>) -> Self {
        let left = Self::from(&e.consequent);
        if left.is_undetermined() {
            return Self::Undetermined;
        }
        let right = Self::from(&e.alternate);
        if left == right {
            return left;
        }
        Self::Undetermined
    }
}

impl<'a> From<&LogicalExpression<'a>> for ValueType {
    fn from(e: &LogicalExpression<'a>) -> Self {
        let left = Self::from(&e.left);
        if !left.is_boolean() {
            return Self::Undetermined;
        }
        let right = Self::from(&e.right);
        if left == right {
            return left;
        }
        Self::Undetermined
    }
}

use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::{One, Zero};
use oxc_ast::ast::{
    match_expression, ArrayExpressionElement, BinaryExpression, Expression, NumericLiteral,
    ObjectProperty, ObjectPropertyKind, PropertyKey, SpreadElement, UnaryExpression,
};
use oxc_semantic::ReferenceFlag;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator, UnaryOperator};

/// Code ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/NodeUtil.java#LL836C6-L836C6)
/// Returns true if this is a literal value. We define a literal value as any node that evaluates
/// to the same thing regardless of when or where it is evaluated. So `/xyz/` and `[3, 5]` are
/// literals, but the name a is not.
///
/// Function literals do not meet this definition, because they lexically capture variables. For
/// example, if you have `function() { return a; }`.
/// If it is evaluated in a different scope, then it captures a different variable. Even if
/// the function did not read any captured variables directly, it would still fail this definition,
/// because it affects the lifecycle of variables in the enclosing scope.
///
/// However, a function literal with respect to a particular scope is a literal.
/// If `include_functions` is true, all function expressions will be treated as literals.
pub trait IsLiteralValue<'a, 'b> {
    fn is_literal_value(&self, include_functions: bool) -> bool;
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for Expression<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::FunctionExpression(_) | Self::ArrowFunctionExpression(_) => include_functions,
            Self::ArrayExpression(expr) => {
                expr.elements.iter().all(|element| element.is_literal_value(include_functions))
            }
            Self::ObjectExpression(expr) => {
                expr.properties.iter().all(|property| property.is_literal_value(include_functions))
            }
            _ => self.is_immutable_value(),
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ArrayExpressionElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_literal_value(include_functions),
            match_expression!(Self) => self.to_expression().is_literal_value(include_functions),
            Self::Elision(_) => true,
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for SpreadElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.argument.is_literal_value(include_functions)
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ObjectPropertyKind<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::ObjectProperty(method) => method.is_literal_value(include_functions),
            Self::SpreadProperty(property) => property.is_literal_value(include_functions),
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ObjectProperty<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.key.is_literal_value(include_functions)
            && self.value.is_literal_value(include_functions)
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for PropertyKey<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::StaticIdentifier(_) | Self::PrivateIdentifier(_) => false,
            match_expression!(Self) => self.to_expression().is_literal_value(include_functions),
        }
    }
}

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
/// Returns true if the node which may have side effects when executed.
/// This version default to the "safe" assumptions when the compiler object
/// is not provided (RegExp have side-effects, etc).
pub trait MayHaveSideEffects<'a, 'b>
where
    Self: CheckForStateChange<'a, 'b>,
{
    fn may_have_side_effects(&self) -> bool {
        self.check_for_state_change(false)
    }
}

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L241)
/// Returns true if some node in n's subtree changes application state. If
/// `check_for_new_objects` is true, we assume that newly created mutable objects (like object
/// literals) change state. Otherwise, we assume that they have no side effects.
pub trait CheckForStateChange<'a, 'b> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool;
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for Expression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::NumericLiteral(_)
            | Self::BooleanLiteral(_)
            | Self::StringLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::NullLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::MetaProperty(_)
            | Self::ThisExpression(_)
            | Self::ClassExpression(_)
            | Self::FunctionExpression(_) => false,
            Self::TemplateLiteral(template) => template
                .expressions
                .iter()
                .any(|expr| expr.check_for_state_change(check_for_new_objects)),
            Self::Identifier(ident) => ident.reference_flag == ReferenceFlag::Write,
            Self::UnaryExpression(unary_expr) => {
                unary_expr.check_for_state_change(check_for_new_objects)
            }
            Self::ParenthesizedExpression(p) => {
                p.expression.check_for_state_change(check_for_new_objects)
            }
            Self::ConditionalExpression(p) => {
                p.test.check_for_state_change(check_for_new_objects)
                    || p.consequent.check_for_state_change(check_for_new_objects)
                    || p.alternate.check_for_state_change(check_for_new_objects)
            }
            Self::SequenceExpression(s) => {
                s.expressions.iter().any(|expr| expr.check_for_state_change(check_for_new_objects))
            }
            Self::BinaryExpression(binary_expr) => {
                binary_expr.check_for_state_change(check_for_new_objects)
            }
            Self::ObjectExpression(object_expr) => {
                if check_for_new_objects {
                    return true;
                }

                object_expr
                    .properties
                    .iter()
                    .any(|property| property.check_for_state_change(check_for_new_objects))
            }
            Self::ArrayExpression(array_expr) => {
                if check_for_new_objects {
                    return true;
                }
                array_expr
                    .elements
                    .iter()
                    .any(|element| element.check_for_state_change(check_for_new_objects))
            }
            _ => true,
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for UnaryExpression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        if is_simple_unary_operator(self.operator) {
            return self.argument.check_for_state_change(check_for_new_objects);
        }
        true
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for BinaryExpression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        let left = self.left.check_for_state_change(check_for_new_objects);
        let right = self.right.check_for_state_change(check_for_new_objects);

        left || right
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ArrayExpressionElement<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.check_for_state_change(check_for_new_objects),
            match_expression!(Self) => {
                self.to_expression().check_for_state_change(check_for_new_objects)
            }
            Self::Elision(_) => false,
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ObjectPropertyKind<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::ObjectProperty(method) => method.check_for_state_change(check_for_new_objects),
            Self::SpreadProperty(spread_element) => {
                spread_element.check_for_state_change(check_for_new_objects)
            }
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for SpreadElement<'a> {
    fn check_for_state_change(&self, _check_for_new_objects: bool) -> bool {
        // Object-rest and object-spread may trigger a getter.
        // TODO: Closure Compiler assumes that getters may side-free when set `assumeGettersArePure`.
        // https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AstAnalyzer.java#L282
        true
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ObjectProperty<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        self.key.check_for_state_change(check_for_new_objects)
            || self.value.check_for_state_change(check_for_new_objects)
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for PropertyKey<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::StaticIdentifier(_) | Self::PrivateIdentifier(_) => false,
            match_expression!(Self) => {
                self.to_expression().check_for_state_change(check_for_new_objects)
            }
        }
    }
}

impl<'a, 'b> MayHaveSideEffects<'a, 'b> for Expression<'a> {}
impl<'a, 'b> MayHaveSideEffects<'a, 'b> for UnaryExpression<'a> {}

/// A "simple" operator is one whose children are expressions, has no direct side-effects.
fn is_simple_unary_operator(operator: UnaryOperator) -> bool {
    operator != UnaryOperator::Delete
}

#[derive(PartialEq)]
pub enum NumberValue {
    Number(f64),
    PositiveInfinity,
    NegativeInfinity,
    NaN,
}

impl NumberValue {
    #[must_use]
    pub fn not(&self) -> Self {
        match self {
            Self::Number(num) => Self::Number(-num),
            Self::PositiveInfinity => Self::NegativeInfinity,
            Self::NegativeInfinity => Self::PositiveInfinity,
            Self::NaN => Self::NaN,
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(self, Self::NaN)
    }
}

impl std::ops::Add<Self> for NumberValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Self::Number(num) => match other {
                Self::Number(other_num) => Self::Number(num + other_num),
                Self::PositiveInfinity => Self::PositiveInfinity,
                Self::NegativeInfinity => Self::NegativeInfinity,
                Self::NaN => Self::NaN,
            },
            Self::NaN => Self::NaN,
            Self::PositiveInfinity => match other {
                Self::NaN | Self::NegativeInfinity => Self::NaN,
                _ => Self::PositiveInfinity,
            },
            Self::NegativeInfinity => match other {
                Self::NaN | Self::PositiveInfinity => Self::NaN,
                _ => Self::NegativeInfinity,
            },
        }
    }
}

impl TryFrom<NumberValue> for f64 {
    type Error = ();

    fn try_from(value: NumberValue) -> Result<Self, Self::Error> {
        match value {
            NumberValue::Number(num) => Ok(num),
            NumberValue::PositiveInfinity => Ok(Self::INFINITY),
            NumberValue::NegativeInfinity => Ok(Self::NEG_INFINITY),
            NumberValue::NaN => Err(()),
        }
    }
}

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/NodeUtil.java#L540)
pub fn get_string_bigint_value(raw_string: &str) -> Option<BigInt> {
    if raw_string.contains('\u{000b}') {
        // vertical tab is not always whitespace
        return None;
    }

    let s = raw_string.trim();

    if s.is_empty() {
        return Some(BigInt::zero());
    }

    if s.len() > 2 && s.starts_with('0') {
        let radix: u32 = match s.chars().nth(1) {
            Some('x' | 'X') => 16,
            Some('o' | 'O') => 8,
            Some('b' | 'B') => 2,
            _ => 0,
        };

        if radix == 0 {
            return None;
        }

        return BigInt::parse_bytes(s[2..].as_bytes(), radix);
    }

    return BigInt::parse_bytes(s.as_bytes(), 10);
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L348)
/// Gets the value of a node as a Number, or None if it cannot be converted.
/// This method does not consider whether `expr` may have side effects.
pub fn get_number_value(expr: &Expression) -> Option<NumberValue> {
    match expr {
        Expression::NumericLiteral(number_literal) => {
            Some(NumberValue::Number(number_literal.value))
        }
        Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
            UnaryOperator::UnaryPlus => get_number_value(&unary_expr.argument),
            UnaryOperator::UnaryNegation => get_number_value(&unary_expr.argument).map(|v| v.not()),
            UnaryOperator::BitwiseNot => get_number_value(&unary_expr.argument).map(|value| {
                match value {
                    NumberValue::Number(num) => {
                        NumberValue::Number(f64::from(!NumericLiteral::ecmascript_to_int32(num)))
                    }
                    // ~Infinity -> -1
                    // ~-Infinity -> -1
                    // ~NaN -> -1
                    _ => NumberValue::Number(-1_f64),
                }
            }),
            UnaryOperator::LogicalNot => get_boolean_value(expr)
                .map(|boolean| if boolean { 1_f64 } else { 0_f64 })
                .map(NumberValue::Number),
            UnaryOperator::Void => Some(NumberValue::NaN),
            _ => None,
        },
        Expression::BooleanLiteral(bool_literal) => {
            if bool_literal.value {
                Some(NumberValue::Number(1.0))
            } else {
                Some(NumberValue::Number(0.0))
            }
        }
        Expression::NullLiteral(_) => Some(NumberValue::Number(0.0)),
        Expression::Identifier(ident) => match ident.name.as_str() {
            "Infinity" => Some(NumberValue::PositiveInfinity),
            "NaN" | "undefined" => Some(NumberValue::NaN),
            _ => None,
        },
        // TODO: will be implemented in next PR, just for test pass now.
        Expression::StringLiteral(string_literal) => string_literal
            .value
            .parse::<f64>()
            .map_or(Some(NumberValue::NaN), |num| Some(NumberValue::Number(num))),
        _ => None,
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn get_bigint_value(expr: &Expression) -> Option<BigInt> {
    match expr {
        Expression::NumericLiteral(number_literal) => {
            let value = number_literal.value;
            if value.abs() < 2_f64.powi(53) && is_exact_int64(value) {
                Some(BigInt::from(value as i64))
            } else {
                None
            }
        }
        Expression::BigIntLiteral(_bigint_literal) => {
            // TODO: evaluate the bigint value
            None
        }
        Expression::BooleanLiteral(bool_literal) => {
            if bool_literal.value {
                Some(BigInt::one())
            } else {
                Some(BigInt::zero())
            }
        }
        Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
            UnaryOperator::LogicalNot => {
                get_boolean_value(expr)
                    .map(|boolean| if boolean { BigInt::one() } else { BigInt::zero() })
            }
            UnaryOperator::UnaryNegation => {
                get_bigint_value(&unary_expr.argument).map(std::ops::Neg::neg)
            }
            UnaryOperator::BitwiseNot => {
                get_bigint_value(&unary_expr.argument).map(std::ops::Not::not)
            }
            UnaryOperator::UnaryPlus => get_bigint_value(&unary_expr.argument),
            _ => None,
        },
        Expression::StringLiteral(string_literal) => get_string_bigint_value(&string_literal.value),
        Expression::TemplateLiteral(_) => {
            get_string_value(expr).and_then(|value| get_string_bigint_value(&value))
        }
        _ => None,
    }
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L104-L114)
/// Returns the number value of the node if it has one and it cannot have side effects.
pub fn get_side_free_number_value(expr: &Expression) -> Option<NumberValue> {
    let value = get_number_value(expr);
    // Calculating the number value, if any, is likely to be faster than calculating side effects,
    // and there are only a very few cases where we can compute a number value, but there could
    // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
    // of `doSomething()`
    if value.is_some() && expr.may_have_side_effects() {
        None
    } else {
        value
    }
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L121)
pub fn get_side_free_bigint_value(expr: &Expression) -> Option<BigInt> {
    let value = get_bigint_value(expr);
    // Calculating the bigint value, if any, is likely to be faster than calculating side effects,
    // and there are only a very few cases where we can compute a bigint value, but there could
    // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
    // of `doSomething()`
    if value.is_some() && expr.may_have_side_effects() {
        None
    } else {
        value
    }
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109)
/// Gets the boolean value of a node that represents an expression, or `None` if no
/// such value can be determined by static analysis.
/// This method does not consider whether the node may have side-effects.
pub fn get_boolean_value(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::RegExpLiteral(_)
        | Expression::ArrayExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::NewExpression(_)
        | Expression::ObjectExpression(_) => Some(true),
        Expression::NullLiteral(_) => Some(false),
        Expression::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
        Expression::NumericLiteral(number_literal) => Some(number_literal.value != 0.0),
        Expression::BigIntLiteral(big_int_literal) => Some(!big_int_literal.is_zero()),
        Expression::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
        Expression::TemplateLiteral(template_literal) => {
            // only for ``
            template_literal
                .quasis
                .first()
                .filter(|quasi| quasi.tail)
                .and_then(|quasi| quasi.value.cooked.as_ref())
                .map(|cooked| !cooked.is_empty())
        }
        Expression::Identifier(ident) => {
            if expr.is_undefined() || ident.name == "NaN" {
                Some(false)
            } else if ident.name == "Infinity" {
                Some(true)
            } else {
                None
            }
        }
        Expression::AssignmentExpression(assign_expr) => {
            match assign_expr.operator {
                AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => None,
                // For ASSIGN, the value is the value of the RHS.
                _ => get_boolean_value(&assign_expr.right),
            }
        }
        Expression::LogicalExpression(logical_expr) => {
            match logical_expr.operator {
                // true && true -> true
                // true && false -> false
                // a && true -> None
                LogicalOperator::And => {
                    let left = get_boolean_value(&logical_expr.left);
                    let right = get_boolean_value(&logical_expr.right);

                    match (left, right) {
                        (Some(true), Some(true)) => Some(true),
                        (Some(false), _) | (_, Some(false)) => Some(false),
                        (None, _) | (_, None) => None,
                    }
                }
                // true || false -> true
                // false || false -> false
                // a || b -> None
                LogicalOperator::Or => {
                    let left = get_boolean_value(&logical_expr.left);
                    let right = get_boolean_value(&logical_expr.right);

                    match (left, right) {
                        (Some(true), _) | (_, Some(true)) => Some(true),
                        (Some(false), Some(false)) => Some(false),
                        (None, _) | (_, None) => None,
                    }
                }
                LogicalOperator::Coalesce => None,
            }
        }
        Expression::SequenceExpression(sequence_expr) => {
            // For sequence expression, the value is the value of the RHS.
            sequence_expr.expressions.last().and_then(get_boolean_value)
        }
        Expression::UnaryExpression(unary_expr) => {
            if unary_expr.operator == UnaryOperator::Void {
                Some(false)
            } else if matches!(
                unary_expr.operator,
                UnaryOperator::BitwiseNot | UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) {
                // ~0 -> true
                // +1 -> true
                // +0 -> false
                // -0 -> false
                get_number_value(expr).map(|value| value != NumberValue::Number(0_f64))
            } else if unary_expr.operator == UnaryOperator::LogicalNot {
                // !true -> false
                get_boolean_value(&unary_expr.argument).map(|boolean| !boolean)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L234)
/// Gets the value of a node as a String, or `None` if it cannot be converted. When it returns a
/// String, this method effectively emulates the `String()` JavaScript cast function.
/// This method does not consider whether `expr` may have side effects.
pub fn get_string_value<'a>(expr: &'a Expression) -> Option<Cow<'a, str>> {
    match expr {
        Expression::StringLiteral(string_literal) => {
            Some(Cow::Borrowed(string_literal.value.as_str()))
        }
        Expression::TemplateLiteral(template_literal) => {
            // TODO: I don't know how to iterate children of TemplateLiteral in order,so only checkout string like `hi`.
            // Closure-compiler do more: [case TEMPLATELIT](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L241-L256).
            template_literal
                .quasis
                .first()
                .filter(|quasi| quasi.tail)
                .and_then(|quasi| quasi.value.cooked.as_ref())
                .map(|cooked| Cow::Borrowed(cooked.as_str()))
        }
        Expression::Identifier(ident) => {
            let name = ident.name.as_str();
            if matches!(name, "undefined" | "Infinity" | "NaN") {
                Some(Cow::Borrowed(name))
            } else {
                None
            }
        }
        Expression::NumericLiteral(number_literal) => {
            Some(Cow::Owned(number_literal.value.to_string()))
        }
        Expression::BigIntLiteral(big_int_literal) => {
            Some(Cow::Owned(big_int_literal.raw.to_string()))
        }
        Expression::NullLiteral(_) => Some(Cow::Borrowed("null")),
        Expression::BooleanLiteral(bool_literal) => {
            if bool_literal.value {
                Some(Cow::Borrowed("true"))
            } else {
                Some(Cow::Borrowed("false"))
            }
        }
        Expression::UnaryExpression(unary_expr) => {
            match unary_expr.operator {
                UnaryOperator::Void => Some(Cow::Borrowed("undefined")),
                UnaryOperator::LogicalNot => {
                    get_boolean_value(&unary_expr.argument).map(|boolean| {
                        // need reversed.
                        if boolean {
                            Cow::Borrowed("false")
                        } else {
                            Cow::Borrowed("true")
                        }
                    })
                }
                _ => None,
            }
        }
        Expression::ArrayExpression(_) => {
            // TODO: https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L302-L303
            None
        }
        Expression::ObjectExpression(_) => Some(Cow::Borrowed("[object Object]")),
        _ => None,
    }
}

/// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L139-L149)
/// Gets the value of a node as a String, or `None` if it cannot be converted.
/// This method effectively emulates the `String()` JavaScript cast function when
/// possible and the node has no side effects. Otherwise, it returns `None`.
pub fn get_side_free_string_value<'a>(expr: &'a Expression) -> Option<Cow<'a, str>> {
    let value = get_string_value(expr);
    // Calculating the string value, if any, is likely to be faster than calculating side effects,
    // and there are only a very few cases where we can compute a string value, but there could
    // also be side effects. e.g. `void doSomething()` has value 'undefined', regardless of the
    // behavior of `doSomething()`
    if value.is_some() && !expr.may_have_side_effects() {
        return value;
    }
    None
}

use oxc_semantic::ReferenceFlag;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator, UnaryOperator};

use crate::hir::{
    ArrayExpressionElement, Expression, NumberLiteral, ObjectProperty, ObjectPropertyKind,
    PropertyKey, SpreadElement, UnaryExpression,
};

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
            Self::FunctionExpression(_) | Self::ArrowExpression(_) => include_functions,
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
            Self::Expression(expr) => expr.is_literal_value(include_functions),
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
            Self::Identifier(_) | Self::PrivateIdentifier(_) => false,
            Self::Expression(expr) => expr.is_literal_value(include_functions),
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
            Self::NumberLiteral(_)
            | Self::BooleanLiteral(_)
            | Self::StringLiteral(_)
            | Self::BigintLiteral(_)
            | Self::NullLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::FunctionExpression(_) => false,
            Self::Identifier(ident) => ident.reference_flag == ReferenceFlag::Write,
            Self::UnaryExpression(unary_expr) => {
                unary_expr.check_for_state_change(check_for_new_objects)
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

impl<'a, 'b> CheckForStateChange<'a, 'b> for ArrayExpressionElement<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.check_for_state_change(check_for_new_objects),
            Self::Expression(expr) => expr.check_for_state_change(check_for_new_objects),
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
            Self::Identifier(_) | Self::PrivateIdentifier(_) => false,
            Self::Expression(expr) => expr.check_for_state_change(check_for_new_objects),
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
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L348)
/// Gets the value of a node as a Number, or None if it cannot be converted.
/// This method does not consider whether `expr` may have side effects.
pub fn get_number_value(expr: &Expression) -> Option<NumberValue> {
    match expr {
        Expression::NumberLiteral(number_literal) => {
            Some(NumberValue::Number(number_literal.value))
        }
        Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
            UnaryOperator::UnaryPlus => get_number_value(&unary_expr.argument),
            UnaryOperator::UnaryNegation => get_number_value(&unary_expr.argument).map(|v| v.not()),
            UnaryOperator::BitwiseNot => get_number_value(&unary_expr.argument).map(|value| {
                match value {
                    NumberValue::Number(num) => {
                        NumberValue::Number(f64::from(!NumberLiteral::ecmascript_to_int32(num)))
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

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L104-L114)
/// Returns the number value of the node if it has one and it cannot have side effects.
pub fn get_side_free_number_value(expr: &Expression) -> Option<NumberValue> {
    let value = get_number_value(expr);
    // Calculating the number value, if any, is likely to be faster than calculating side effects,
    // and there are only a very few cases where we can compute a number value, but there could
    // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
    // of `doSomething()`
    if value.is_some() && !expr.may_have_side_effects() {
        return value;
    }
    None
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109)
/// Gets the boolean value of a node that represents an expression, or `None` if no
/// such value can be determined by static analysis.
/// This method does not consider whether the node may have side-effects.
pub fn get_boolean_value(expr: &Expression) -> Option<bool> {
    use num_traits::Zero;

    match expr {
        Expression::RegExpLiteral(_)
        | Expression::ArrayExpression(_)
        | Expression::ArrowExpression(_)
        | Expression::ClassExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::NewExpression(_)
        | Expression::ObjectExpression(_) => Some(true),
        Expression::NullLiteral(_) => Some(false),
        Expression::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
        Expression::NumberLiteral(number_literal) => Some(number_literal.value != 0.0),
        Expression::BigintLiteral(big_int_literal) => Some(!big_int_literal.value.is_zero()),
        Expression::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
        Expression::TemplateLiteral(template_literal) => {
            // only for ``
            if let Some(quasi) = template_literal.quasis.get(0) && quasi.tail {
                Some(quasi.value.cooked.as_ref().map_or(false, |cooked| !cooked.is_empty()))
            } else {
                None
            }
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
            let predict = |expr: &&Expression| get_boolean_value(expr) == Some(true);
            match logical_expr.operator {
                // true && true -> true
                // true && false -> false
                LogicalOperator::And => {
                    Some([&logical_expr.left, &logical_expr.right].iter().all(predict))
                }
                // true || false -> true
                // false || false -> false
                LogicalOperator::Or => {
                    Some([&logical_expr.left, &logical_expr.right].iter().any(predict))
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

use oxc_syntax::operator::UnaryOperator;

use crate::hir::{
    ArrayExpressionElement, Expression, ObjectProperty, ObjectPropertyKind, PropertyKey,
    SpreadElement,
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
            Self::NumberLiteral(_) => false,
            Self::UnaryExpression(unary_expr) => {
                if is_simple_unary_operator(unary_expr.operator) {
                    return unary_expr.argument.check_for_state_change(check_for_new_objects);
                }

                true
            }
            _ => true,
        }
    }
}

impl<'a, 'b> MayHaveSideEffects<'a, 'b> for Expression<'a> {}

fn is_simple_unary_operator(operator: UnaryOperator) -> bool {
    operator != UnaryOperator::Delete
}

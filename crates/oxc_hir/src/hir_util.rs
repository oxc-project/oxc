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

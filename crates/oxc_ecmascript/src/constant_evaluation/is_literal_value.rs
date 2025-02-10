use oxc_ast::ast::*;

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

pub fn is_immutable_value(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(lit) if lit.is_no_substitution_template() => true,
        Expression::Identifier(ident) => {
            matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
        }
        Expression::UnaryExpression(e)
            if matches!(
                e.operator,
                UnaryOperator::Void | UnaryOperator::LogicalNot | UnaryOperator::UnaryNegation
            ) =>
        {
            is_immutable_value(&e.argument)
        }
        // Operations on bigint can result type error.
        // Expression::BigIntLiteral(_) => false,
        _ => false,
    }
}

impl<'a> IsLiteralValue<'a, '_> for Expression<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::FunctionExpression(_) | Self::ArrowFunctionExpression(_) => include_functions,
            Self::ArrayExpression(expr) => {
                expr.elements.iter().all(|element| element.is_literal_value(include_functions))
            }
            Self::ObjectExpression(expr) => {
                expr.properties.iter().all(|property| property.is_literal_value(include_functions))
            }
            _ => is_immutable_value(self),
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for ArrayExpressionElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_literal_value(include_functions),
            match_expression!(Self) => self.to_expression().is_literal_value(include_functions),
            Self::Elision(_) => true,
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for SpreadElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.argument.is_literal_value(include_functions)
    }
}

impl<'a> IsLiteralValue<'a, '_> for ObjectPropertyKind<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::ObjectProperty(method) => method.is_literal_value(include_functions),
            Self::SpreadProperty(property) => property.is_literal_value(include_functions),
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for ObjectProperty<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.key.is_literal_value(include_functions)
            && self.value.is_literal_value(include_functions)
    }
}

impl<'a> IsLiteralValue<'a, '_> for PropertyKey<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::StaticIdentifier(_) => true,
            Self::PrivateIdentifier(_) => false,
            match_expression!(Self) => self.to_expression().is_literal_value(include_functions),
        }
    }
}

use oxc_ast::ast::ExpressionKind;
use oxc_ast::ast::*;

use crate::GlobalContext;

/// Returns true if this is a literal value.
///
/// We define a literal value as any node that evaluates
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
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool;
}

impl<'a> IsLiteralValue<'a, '_> for Expression<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self.kind() {
            ExpressionKind::BooleanLiteral(_)
            | ExpressionKind::NullLiteral(_)
            | ExpressionKind::NumericLiteral(_)
            | ExpressionKind::BigIntLiteral(_)
            | ExpressionKind::RegExpLiteral(_)
            | ExpressionKind::StringLiteral(_) => true,
            ExpressionKind::TemplateLiteral(lit) => lit.is_literal_value(include_functions, ctx),
            ExpressionKind::Identifier(ident) => {
                matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
                    && ctx.is_global_reference(ident)
            }
            ExpressionKind::ArrayExpression(expr) => expr.is_literal_value(include_functions, ctx),
            ExpressionKind::ObjectExpression(expr) => expr.is_literal_value(include_functions, ctx),
            ExpressionKind::FunctionExpression(_) | ExpressionKind::ArrowFunctionExpression(_) => {
                include_functions
            }
            ExpressionKind::UnaryExpression(e) => e.is_literal_value(include_functions, ctx),
            ExpressionKind::BinaryExpression(e) => e.is_literal_value(include_functions, ctx),
            ExpressionKind::LogicalExpression(e) => {
                e.left.is_literal_value(include_functions, ctx)
                    && e.right.is_literal_value(include_functions, ctx)
            }
            ExpressionKind::ConditionalExpression(e) => {
                e.test.is_literal_value(include_functions, ctx)
                    && e.consequent.is_literal_value(include_functions, ctx)
                    && e.alternate.is_literal_value(include_functions, ctx)
            }
            ExpressionKind::ParenthesizedExpression(e) => {
                e.expression.is_literal_value(include_functions, ctx)
            }
            ExpressionKind::SequenceExpression(e) => {
                e.expressions.iter().all(|expr| expr.is_literal_value(include_functions, ctx))
            }
            _ => false,
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for TemplateLiteral<'a> {
    fn is_literal_value(&self, _include_functions: bool, _ctx: &impl GlobalContext<'a>) -> bool {
        self.is_no_substitution_template()
    }
}

impl<'a> IsLiteralValue<'a, '_> for ArrayExpression<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        self.elements.iter().all(|element| element.is_literal_value(include_functions, ctx))
    }
}

impl<'a> IsLiteralValue<'a, '_> for ObjectExpression<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        self.properties.iter().all(|property| property.is_literal_value(include_functions, ctx))
    }
}

impl<'a> IsLiteralValue<'a, '_> for UnaryExpression<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot | UnaryOperator::Typeof => {
                self.argument.is_literal_value(include_functions, ctx)
            }
            UnaryOperator::UnaryPlus => {
                can_convert_to_number_transparently(&self.argument, include_functions, ctx)
            }
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                can_convert_to_number_transparently(&self.argument, include_functions, ctx)
                    || self.argument.is_big_int_literal()
            }
            UnaryOperator::Delete => false,
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for BinaryExpression<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self.operator {
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                self.left.is_literal_value(include_functions, ctx)
                    && self.right.is_literal_value(include_functions, ctx)
            }
            BinaryOperator::Addition => {
                if (is_immutable_string(&self.left, include_functions, ctx)
                    && can_convert_to_string_transparently(&self.right, include_functions, ctx))
                    || (is_immutable_string(&self.right, include_functions, ctx)
                        && can_convert_to_string_transparently(&self.left, include_functions, ctx))
                {
                    return true;
                }
                (self.left.is_numeric_literal() && self.right.is_numeric_literal())
                    | (self.left.is_big_int_literal() && self.right.is_big_int_literal())
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd => {
                if (self.left.is_numeric_literal()
                    && can_convert_to_number_transparently(&self.right, include_functions, ctx))
                    || (self.right.is_numeric_literal()
                        && can_convert_to_number_transparently(&self.left, include_functions, ctx))
                {
                    return true;
                }
                let (Some(left_big_int), Some(right)) =
                    (self.left.as_big_int_literal(), self.right.as_big_int_literal())
                else {
                    return false;
                };
                let _ = left_big_int;
                // 1n / 0n, 1n % 0n, 1n ** (-1n) throws an error
                match self.operator {
                    BinaryOperator::ShiftRightZeroFill => false,
                    BinaryOperator::Exponential => !right.is_negative(),
                    BinaryOperator::Division | BinaryOperator::Remainder => !right.is_zero(),
                    _ => true,
                }
            }
            BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::In
            | BinaryOperator::Instanceof => false,
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for ArrayExpressionElement<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self {
            // spread element triggers `Symbol.iterator` call
            Self::SpreadElement(_) => false,
            Self::Elision(_) => true,
            match_expression!(Self) => {
                self.to_expression().is_literal_value(include_functions, ctx)
            }
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for ObjectPropertyKind<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self {
            Self::ObjectProperty(property) => property.is_literal_value(include_functions, ctx),
            Self::SpreadProperty(property) => match property.argument.kind() {
                ExpressionKind::ArrayExpression(expr) => {
                    expr.is_literal_value(include_functions, ctx)
                }
                ExpressionKind::StringLiteral(_) => true,
                ExpressionKind::TemplateLiteral(lit) => {
                    lit.is_literal_value(include_functions, ctx)
                }
                ExpressionKind::ObjectExpression(expr) => {
                    expr.is_literal_value(include_functions, ctx)
                }
                _ => false,
            },
        }
    }
}

impl<'a> IsLiteralValue<'a, '_> for ObjectProperty<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        self.key.is_literal_value(include_functions, ctx)
            && self.value.is_literal_value(include_functions, ctx)
    }
}

impl<'a> IsLiteralValue<'a, '_> for PropertyKey<'a> {
    fn is_literal_value(&self, include_functions: bool, ctx: &impl GlobalContext<'a>) -> bool {
        match self {
            Self::StaticIdentifier(_) => true,
            Self::PrivateIdentifier(_) => false,
            match_expression!(Self) => {
                can_convert_to_string_transparently(&self.to_expression(), include_functions, ctx)
            }
        }
    }
}

fn can_convert_to_number_transparently<'a>(
    expr: &Expression<'a>,
    include_functions: bool,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    match expr.kind() {
        ExpressionKind::NumericLiteral(_)
        | ExpressionKind::NullLiteral(_)
        | ExpressionKind::BooleanLiteral(_)
        | ExpressionKind::StringLiteral(_) => true,
        ExpressionKind::TemplateLiteral(lit) => lit.is_literal_value(include_functions, ctx),
        ExpressionKind::Identifier(ident) => {
            matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
                && ctx.is_global_reference(ident)
        }
        ExpressionKind::ArrowFunctionExpression(_) | ExpressionKind::FunctionExpression(_) => {
            include_functions
        }
        ExpressionKind::UnaryExpression(e) => match e.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot | UnaryOperator::Typeof => {
                e.argument.is_literal_value(include_functions, ctx)
            }
            UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                can_convert_to_number_transparently(&e.argument, include_functions, ctx)
            }
            UnaryOperator::Delete => false,
        },
        ExpressionKind::BinaryExpression(e) => match e.operator {
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                e.left.is_literal_value(include_functions, ctx)
                    && e.right.is_literal_value(include_functions, ctx)
            }
            BinaryOperator::Addition => {
                if (is_immutable_string(&e.left, include_functions, ctx)
                    && can_convert_to_string_transparently(&e.right, include_functions, ctx))
                    || (is_immutable_string(&e.right, include_functions, ctx)
                        && can_convert_to_string_transparently(&e.left, include_functions, ctx))
                {
                    return true;
                }
                (e.left.is_numeric_literal() && e.right.is_numeric_literal())
                    | (e.left.is_big_int_literal() && e.right.is_big_int_literal())
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd => {
                if (e.left.is_numeric_literal()
                    && can_convert_to_number_transparently(&e.right, include_functions, ctx))
                    || (e.right.is_numeric_literal()
                        && can_convert_to_number_transparently(&e.left, include_functions, ctx))
                {
                    return true;
                }
                false
            }
            BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::In
            | BinaryOperator::Instanceof => false,
        },
        ExpressionKind::LogicalExpression(e) => {
            can_convert_to_number_transparently(&e.left, include_functions, ctx)
                && can_convert_to_number_transparently(&e.right, include_functions, ctx)
        }
        ExpressionKind::ConditionalExpression(e) => {
            e.test.is_literal_value(include_functions, ctx)
                && can_convert_to_number_transparently(&e.consequent, include_functions, ctx)
                && can_convert_to_number_transparently(&e.alternate, include_functions, ctx)
        }
        ExpressionKind::ParenthesizedExpression(e) => {
            can_convert_to_number_transparently(&e.expression, include_functions, ctx)
        }
        ExpressionKind::SequenceExpression(e) => {
            can_convert_to_number_transparently(
                e.expressions.last().expect("should have at least one element"),
                include_functions,
                ctx,
            ) && e
                .expressions
                .iter()
                .rev()
                .skip(1)
                .all(|expr| expr.is_literal_value(include_functions, ctx))
        }
        _ => false,
    }
}

fn can_convert_to_string_transparently<'a>(
    expr: &Expression<'a>,
    include_functions: bool,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    match expr.kind() {
        ExpressionKind::NumericLiteral(_)
        | ExpressionKind::StringLiteral(_)
        | ExpressionKind::NullLiteral(_)
        | ExpressionKind::BooleanLiteral(_)
        | ExpressionKind::BigIntLiteral(_) => true,
        ExpressionKind::TemplateLiteral(lit) => lit.is_literal_value(include_functions, ctx),
        ExpressionKind::Identifier(ident) => {
            matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
                && ctx.is_global_reference(ident)
        }
        ExpressionKind::ArrowFunctionExpression(_) | ExpressionKind::FunctionExpression(_) => {
            include_functions
        }
        ExpressionKind::UnaryExpression(e) => e.is_literal_value(include_functions, ctx),
        ExpressionKind::BinaryExpression(e) => e.is_literal_value(include_functions, ctx),
        ExpressionKind::LogicalExpression(e) => {
            e.left.is_literal_value(include_functions, ctx)
                && e.right.is_literal_value(include_functions, ctx)
        }
        ExpressionKind::ConditionalExpression(e) => {
            e.test.is_literal_value(include_functions, ctx)
                && can_convert_to_string_transparently(&e.consequent, include_functions, ctx)
                && can_convert_to_string_transparently(&e.alternate, include_functions, ctx)
        }
        ExpressionKind::ParenthesizedExpression(e) => {
            can_convert_to_string_transparently(&e.expression, include_functions, ctx)
        }
        ExpressionKind::SequenceExpression(e) => {
            can_convert_to_string_transparently(
                e.expressions.last().expect("should have at least one element"),
                include_functions,
                ctx,
            ) && e
                .expressions
                .iter()
                .rev()
                .skip(1)
                .all(|expr| expr.is_literal_value(include_functions, ctx))
        }
        _ => false,
    }
}

fn is_immutable_string<'a>(
    expr: &Expression<'a>,
    include_functions: bool,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    match expr.kind() {
        ExpressionKind::StringLiteral(_) => true,
        ExpressionKind::TemplateLiteral(lit) => lit.is_literal_value(include_functions, ctx),
        _ => false,
    }
}

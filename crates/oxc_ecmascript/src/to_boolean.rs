use oxc_ast::ast::Expression;

use crate::GlobalContext;

/// `ToBoolean`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toboolean>
pub trait ToBoolean<'a> {
    fn to_boolean(&self, ctx: &impl GlobalContext<'a>) -> Option<bool>;
}

impl<'a> ToBoolean<'a> for Expression<'a> {
    fn to_boolean(&self, ctx: &impl GlobalContext<'a>) -> Option<bool> {
        // 1. If argument is a Boolean, return argument.
        // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false.
        // 3. NOTE: This step is replaced in section B.3.6.1.
        // 4. Return true.
        match self {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "undefined" if ctx.is_global_reference(ident) == Some(true) => Some(false),
                "Infinity" if ctx.is_global_reference(ident) == Some(true) => Some(true),
                _ => None,
            },
            Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::NewExpression(_)
            | Expression::ObjectExpression(_) => Some(true),
            Expression::NullLiteral(_) => Some(false),
            Expression::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
            Expression::NumericLiteral(lit) => {
                Some(if lit.value.is_nan() { false } else { lit.value != 0.0 })
            }
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
            Expression::SequenceExpression(e) => {
                e.expressions.last().and_then(|expr| expr.to_boolean(ctx))
            }
            _ => None,
        }
    }
}

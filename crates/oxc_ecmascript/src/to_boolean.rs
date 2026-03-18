use oxc_ast::ast::{Expression, ExpressionKind};

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
        match self.kind() {
            ExpressionKind::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "undefined" if ctx.is_global_reference(ident) => Some(false),
                "Infinity" if ctx.is_global_reference(ident) => Some(true),
                _ => None,
            },
            ExpressionKind::RegExpLiteral(_)
            | ExpressionKind::ArrayExpression(_)
            | ExpressionKind::ArrowFunctionExpression(_)
            | ExpressionKind::ClassExpression(_)
            | ExpressionKind::FunctionExpression(_)
            | ExpressionKind::NewExpression(_)
            | ExpressionKind::ObjectExpression(_) => Some(true),
            ExpressionKind::NullLiteral(_) => Some(false),
            ExpressionKind::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
            ExpressionKind::NumericLiteral(lit) => {
                Some(if lit.value.is_nan() { false } else { lit.value != 0.0 })
            }
            ExpressionKind::BigIntLiteral(big_int_literal) => Some(!big_int_literal.is_zero()),
            ExpressionKind::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
            ExpressionKind::TemplateLiteral(template_literal) => {
                // only for ``
                template_literal
                    .quasis
                    .first()
                    .filter(|quasi| quasi.tail)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|cooked| !cooked.is_empty())
            }
            ExpressionKind::SequenceExpression(e) => {
                e.expressions.last().and_then(|expr| expr.to_boolean(ctx))
            }
            _ => None,
        }
    }
}

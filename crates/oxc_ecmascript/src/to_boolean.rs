use oxc_ast::ast::Expression;

/// `ToBoolean`
///
/// <https://tc39.es/ecma262/#sec-toboolean>
pub trait ToBoolean<'a> {
    fn to_boolean(&self) -> Option<bool>;
}

impl<'a> ToBoolean<'a> for Expression<'a> {
    fn to_boolean(&self) -> Option<bool> {
        // 1. If argument is a Boolean, return argument.
        // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false.
        // 3. NOTE: This step is replaced in section B.3.6.1.
        // 4. Return true.
        match self {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "undefined" => Some(false),
                "Infinity" => Some(true),
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
            _ => None,
        }
    }
}

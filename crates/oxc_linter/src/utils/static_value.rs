use oxc_allocator::Allocator;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_str::{JSStr, JSStrBuilder};

/// Resolve a side-effect-free string expression made from string literals, template literals,
/// and `+` concatenation. Returns `None` when any part cannot be determined statically.
///
/// A plain string literal is borrowed; a concatenation is built in `allocator`. Lone
/// surrogates are preserved, and a pair split across parts is joined.
pub fn static_string_value<'a>(
    expression: &Expression<'a>,
    allocator: &'a Allocator,
) -> Option<JSStr<'a>> {
    if let Expression::StringLiteral(literal) = expression.get_inner_expression() {
        return Some(literal.value);
    }
    let mut builder = JSStrBuilder::new_in(allocator);
    push_static_string_value(expression, &mut builder)?;
    Some(builder.into_js_str())
}

fn push_static_string_value(
    expression: &Expression<'_>,
    builder: &mut JSStrBuilder<'_>,
) -> Option<()> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => builder.push_js_str(literal.value),
        Expression::TemplateLiteral(template) => {
            for (index, quasi) in template.quasis.iter().enumerate() {
                builder.push_js_str(quasi.value.cooked?);
                if let Some(expr) = template.expressions.get(index) {
                    push_static_string_value(expr, builder)?;
                }
            }
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            push_static_string_value(&binary.left, builder)?;
            push_static_string_value(&binary.right, builder)?;
        }
        _ => return None,
    }
    Some(())
}

use oxc_ast::ast::{BinaryOperator, Expression};

/// Resolve a side-effect-free string expression made from string literals, template literals,
/// and `+` concatenation. Returns `None` when any part cannot be determined statically.
pub fn static_string_value(expression: &Expression<'_>) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => {
            let mut value = String::new();
            for (index, quasi) in template.quasis.iter().enumerate() {
                value.push_str(quasi.value.cooked.as_ref()?);
                if let Some(expr) = template.expressions.get(index) {
                    value.push_str(&static_string_value(expr)?);
                }
            }
            Some(value)
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let mut value = static_string_value(&binary.left)?;
            value.push_str(&static_string_value(&binary.right)?);
            Some(value)
        }
        _ => None,
    }
}

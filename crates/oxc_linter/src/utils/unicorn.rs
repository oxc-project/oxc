use oxc_ast::ast::Expression;

pub fn is_node_value(expr: &Expression) -> bool {
    return !matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::ArrowExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TemplateLiteral(_)
    );
}

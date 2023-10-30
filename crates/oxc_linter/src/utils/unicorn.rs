use oxc_ast::ast::Expression;

pub fn is_dom_node_call(expr: &Expression) -> bool {
    !matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::ArrowExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::StringLiteral(_)
    )
}

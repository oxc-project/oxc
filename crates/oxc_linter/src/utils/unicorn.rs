use oxc_ast::ast::{Expression, Statement};

pub fn is_node_value_not_dom_node(expr: &Expression) -> bool {
    matches!(
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

pub fn is_empty_stmt(stmt: &Statement) -> bool {
    match stmt {
        Statement::BlockStatement(block_stmt) => {
            if block_stmt.body.is_empty() || block_stmt.body.iter().all(|node| is_empty_stmt(node))
            {
                return true;
            }
            false
        }
        Statement::EmptyStatement(_) => true,
        _ => false,
    }
}

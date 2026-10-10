use oxc_ast::ast::{CallExpression, Expression};

pub fn call_uses_optional_chain(call_expr: &CallExpression) -> bool {
    call_expr.optional || expression_uses_optional_chain(&call_expr.callee)
}

pub fn expression_uses_optional_chain(expr: &Expression) -> bool {
    let expr = expr.get_inner_expression();

    if matches!(expr, Expression::ChainExpression(_)) {
        return true;
    }

    if let Some(member_expr) = expr.as_member_expression() {
        return member_expr.optional() || expression_uses_optional_chain(member_expr.object());
    }

    if let Expression::CallExpression(call_expr) = expr {
        return call_expr.optional || expression_uses_optional_chain(&call_expr.callee);
    }

    false
}

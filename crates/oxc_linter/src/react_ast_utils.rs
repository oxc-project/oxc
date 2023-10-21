use oxc_ast::ast::CallExpression;

pub fn is_create_element_call(call_expr: &CallExpression) -> bool {
    if let Some(member_expr) = call_expr.callee.get_member_expr() {
        return member_expr.static_property_name() == Some("createElement");
    }

    false
}

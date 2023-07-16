use oxc_hir::hir::Expression;

pub(super) fn is_console(expr: &Expression<'_>) -> bool {
    // let Statement::ExpressionStatement(expr) = stmt else { return false };
    let Expression::CallExpression(call_expr) = &expr else { return false };
    let Expression::MemberExpression(member_expr) = &call_expr.callee else { return false };
    let obj = member_expr.object();
    let Some(ident) = obj.get_identifier_reference() else { return false };
    ident.name == "console"
}

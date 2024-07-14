use oxc_ast::{ast::CallExpression, AstKind};
use oxc_semantic::AstNode;

use crate::LintContext;

pub fn is_promise_callback(call_expr: &CallExpression) -> bool {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Some(prop_name) = member_expr.static_property_name() else {
        return false;
    };

    if !matches!(prop_name, "catch" | "then") {
        return false;
    }

    true
}

pub fn is_inside_promise<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // kind: Argument(FunctionExpression)
    let Some(func_argument_expr) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };

    let Some(call_expr_node) = ctx.nodes().parent_node(func_argument_expr.id()) else {
        return false;
    };

    let AstKind::CallExpression(call_expr) = call_expr_node.kind() else {
        return false;
    };

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Some(prop_name) = member_expr.static_property_name() else {
        return false;
    };

    if !matches!(prop_name, "catch" | "then") {
        return false;
    }

    true
}

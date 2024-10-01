use oxc_ast::{
    ast::{CallExpression, ConditionalExpression, Expression},
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

use super::is_logical_expression;
use crate::{ast_util::outermost_paren_parent, LintContext};
pub fn is_logic_not(node: &AstKind) -> bool {
    matches!(node, AstKind::UnaryExpression(unary_expr) if unary_expr.operator == UnaryOperator::LogicalNot)
}
fn is_logic_not_argument<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };
    is_logic_not(&parent.kind())
}
pub fn is_boolean_call(kind: &AstKind) -> bool {
    matches!(
        kind,
        AstKind::CallExpression(CallExpression {
            callee: Expression::Identifier(ident),
            arguments,
            ..
        }) if ident.name == "Boolean" && arguments.len() == 1
    )
}
pub fn is_boolean_call_argument<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let arg_id = ctx.nodes().parent_id(node.id());
    let parent = arg_id.and_then(|id| ctx.nodes().parent_kind(id));
    // println!("{parent:#?}");
    matches!(parent, Some(parent) if is_boolean_call(&parent))
}

pub fn is_boolean_node<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let kind = node.kind();

    if is_logic_not(&kind)
        || is_logic_not_argument(node, ctx)
        || is_boolean_call(&kind)
        || is_boolean_call_argument(node, ctx)
    {
        return true;
    }

    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };

    if matches!(
        parent.kind(),
        AstKind::IfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ForStatement(_)
    ) {
        return true;
    }

    if let AstKind::ConditionalExpression(ConditionalExpression {
        test: conditional_test, ..
    }) = parent.kind()
    {
        let expr_span = conditional_test.get_inner_expression().without_parentheses().span();
        return expr_span == node.kind().span();
    }

    if is_logical_expression(parent) {
        return is_boolean_node(parent, ctx);
    }

    false
}

pub fn get_boolean_ancestor<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
    // (node, is_negative)
) -> (&'b AstNode<'a>, bool) {
    let mut is_negative = false;
    let mut cur = node;
    loop {
        if let Some(parent) = outermost_paren_parent(cur, ctx) {
            let kind = parent.kind();
            if is_logic_not(&kind) {
                is_negative = !is_negative;
                cur = parent;
                continue;
            }
            if let Some(parent) = ctx.nodes().parent_node(parent.id()) {
                if is_boolean_call(&parent.kind()) {
                    cur = parent;
                    continue;
                }
            }
            break;
        }
        break;
    }
    (cur, is_negative)
}

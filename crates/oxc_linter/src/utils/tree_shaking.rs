use oxc_ast::{ast::Expression, AstKind};
use oxc_semantic::AstNodeId;

use crate::LintContext;

#[allow(dead_code)]
pub enum Value {
    Boolean(bool),
    Number(f64),
}

pub fn get_write_expr<'a, 'b>(
    node_id: AstNodeId,
    ctx: &'b LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let parent = ctx.nodes().parent_kind(node_id)?;
    match parent {
        AstKind::AssignmentExpression(assign_expr) => Some(&assign_expr.right),
        _ => None,
    }
}

pub fn no_effects() {}

use oxc_ast::{ast::Expression, AstKind};
use oxc_semantic::AstNodeId;
use oxc_span::Span;

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

/// Comments containing @__PURE__ or #__PURE__ mark a specific function call
/// or constructor invocation as side effect free.
///
/// Such an annotation is considered valid if it directly
/// precedes a function call or constructor invocation
/// and is only separated from the callee by white-space or comments.
///
/// The only exception are parentheses that wrap a call or invocation.
///
/// <https://rollupjs.org/configuration-options/#pure>
pub fn has_pure_notation(span: Span, ctx: &LintContext) -> bool {
    let Some((start, comment)) = ctx.semantic().trivias().comments_range(..span.start).next_back()
    else {
        return false;
    };
    let span = Span::new(*start, comment.end);
    let raw = span.source_text(ctx.semantic().source_text());

    raw.contains("@__PURE__") || raw.contains("#__PURE__")
}

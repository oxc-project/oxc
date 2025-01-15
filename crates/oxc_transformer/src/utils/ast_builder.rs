use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

/// `object` -> `object.call`.
pub(crate) fn create_member_callee<'a>(
    object: Expression<'a>,
    property: &'static str,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, Atom::from(property));
    Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
}

/// `object` -> `object.bind(this)`.
pub(crate) fn create_bind_call<'a>(
    callee: Expression<'a>,
    this: Expression<'a>,
    span: Span,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = create_member_callee(callee, "bind", ctx);
    let arguments = ctx.ast.vec1(Argument::from(this));
    ctx.ast.expression_call(span, callee, NONE, arguments, false)
}

/// `object` -> `object.call(...arguments)`.
pub(crate) fn create_call_call<'a>(
    callee: Expression<'a>,
    this: Expression<'a>,
    span: Span,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = create_member_callee(callee, "call", ctx);
    let arguments = ctx.ast.vec1(Argument::from(this));
    ctx.ast.expression_call(span, callee, NONE, arguments, false)
}

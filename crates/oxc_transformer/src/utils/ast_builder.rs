use oxc_ast::{ast::*, NONE};
use oxc_semantic::ScopeFlags;
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

/// Wrap the expression with an arrow function iife.
///
/// `expr` ->  `(() => { return expr; })()`
pub(crate) fn wrap_arrow_function_iife<'a>(
    expr: &mut Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let scope_id =
        ctx.insert_scope_below_expression(expr, ScopeFlags::Arrow | ScopeFlags::Function);

    let kind = FormalParameterKind::ArrowFormalParameters;
    let params = ctx.ast.formal_parameters(SPAN, kind, ctx.ast.vec(), NONE);
    let statements =
        ctx.ast.vec1(ctx.ast.statement_return(SPAN, Some(ctx.ast.move_expression(expr))));
    let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), statements);
    let arrow = ctx.ast.alloc_arrow_function_expression_with_scope_id(
        SPAN, false, false, NONE, params, NONE, body, scope_id,
    );
    // IIFE
    ctx.ast.expression_call(
        SPAN,
        Expression::ArrowFunctionExpression(arrow),
        NONE,
        ctx.ast.vec(),
        false,
    )
}

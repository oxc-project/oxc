use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{ast::*, NONE};
use oxc_semantic::{ScopeFlags, ScopeId};
use oxc_span::{GetSpan, SPAN};
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

/// Wrap an `Expression` in an arrow function IIFE (immediately invoked function expression)
/// with a body block.
///
/// `expr` -> `(() => { return expr; })()`
pub(crate) fn wrap_expression_in_arrow_function_iife<'a>(
    expr: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let scope_id =
        ctx.insert_scope_below_expression(&expr, ScopeFlags::Arrow | ScopeFlags::Function);
    let span = expr.span();
    let stmts = ctx.ast.vec1(ctx.ast.statement_return(SPAN, Some(expr)));
    wrap_statements_in_arrow_function_iife(stmts, scope_id, span, ctx)
}

/// Wrap statements in an IIFE (immediately invoked function expression).
///
/// `x; y; z;` -> `(() => { x; y; z; })()`
pub(crate) fn wrap_statements_in_arrow_function_iife<'a>(
    stmts: ArenaVec<'a, Statement<'a>>,
    scope_id: ScopeId,
    span: Span,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let kind = FormalParameterKind::ArrowFormalParameters;
    let params = ctx.ast.alloc_formal_parameters(SPAN, kind, ctx.ast.vec(), NONE);
    let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), stmts);
    let arrow =
        Expression::ArrowFunctionExpression(ctx.ast.alloc_arrow_function_expression_with_scope_id(
            SPAN, false, false, NONE, params, NONE, body, scope_id,
        ));
    ctx.ast.expression_call(span, arrow, NONE, ctx.ast.vec(), false)
}

/// `object` -> `object.prototype`.
pub(crate) fn create_prototype_member<'a>(
    object: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, Atom::from("prototype"));
    let static_member = ctx.ast.member_expression_static(SPAN, object, property, false);
    Expression::from(static_member)
}

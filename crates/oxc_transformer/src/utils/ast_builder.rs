use std::iter;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

/// `object` -> `object.call`.
pub fn create_member_callee<'a>(
    object: Expression<'a>,
    property: &'static str,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, Atom::from(property));
    Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
}

/// `object` -> `object.bind(this)`.
pub fn create_bind_call<'a>(
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
pub fn create_call_call<'a>(
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
pub fn wrap_expression_in_arrow_function_iife<'a>(
    expr: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let scope_id =
        ctx.insert_scope_below_expression(&expr, ScopeFlags::Arrow | ScopeFlags::Function);
    let span = expr.span();
    let stmt = ctx.ast.statement_return(SPAN, Some(expr));
    let stmts = ctx.ast.vec1(stmt);
    wrap_statements_in_arrow_function_iife(stmts, scope_id, span, ctx)
}

/// Wrap statements in an IIFE (immediately invoked function expression).
///
/// `x; y; z;` -> `(() => { x; y; z; })()`
pub fn wrap_statements_in_arrow_function_iife<'a>(
    stmts: ArenaVec<'a, Statement<'a>>,
    scope_id: ScopeId,
    span: Span,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let kind = FormalParameterKind::ArrowFormalParameters;
    let items = ctx.ast.vec();
    let params = ctx.ast.alloc_formal_parameters(SPAN, kind, items, NONE);
    let directives = ctx.ast.vec();
    let body = ctx.ast.alloc_function_body(SPAN, directives, stmts);
    let arrow = ctx.ast.expression_arrow_function_with_scope_id_and_pure_and_pife(
        SPAN, false, false, NONE, params, NONE, body, scope_id, false, false,
    );
    let arguments = ctx.ast.vec();
    ctx.ast.expression_call(span, arrow, NONE, arguments, false)
}

/// `object` -> `object.prototype`.
pub fn create_prototype_member<'a>(
    object: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, Atom::from("prototype"));
    let static_member = ctx.ast.member_expression_static(SPAN, object, property, false);
    Expression::from(static_member)
}

/// `object` -> `object.a`.
pub fn create_property_access<'a>(
    span: Span,
    object: Expression<'a>,
    property: &str,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, ctx.ast.atom(property));
    Expression::from(ctx.ast.member_expression_static(span, object, property, false))
}

/// `this.property`
#[inline]
pub fn create_this_property_access<'a>(
    span: Span,
    property: Atom<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> MemberExpression<'a> {
    let object = ctx.ast.expression_this(span);
    let property = ctx.ast.identifier_name(SPAN, property);
    ctx.ast.member_expression_static(span, object, property, false)
}

/// `this.property`
#[inline]
pub fn create_this_property_assignment<'a>(
    span: Span,
    property: Atom<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> AssignmentTarget<'a> {
    AssignmentTarget::from(create_this_property_access(span, property, ctx))
}

/// Create assignment to a binding.
pub fn create_assignment<'a>(
    binding: &BoundIdentifier<'a>,
    value: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let target = binding.create_target(ReferenceFlags::Write, ctx);
    ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, target, value)
}

/// `super(...args);`
pub fn create_super_call<'a>(
    args_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let read_expr = args_binding.create_read_expression(ctx);
    let spread_element = ctx.ast.argument_spread_element(SPAN, read_expr);
    let arguments = ctx.ast.vec1(spread_element);
    let callee = ctx.ast.expression_super(SPAN);
    ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
}

/// * With super class:
///   `constructor(..._args) { super(..._args); statements }`
/// * Without super class:
//    `constructor() { statements }`
pub fn create_class_constructor<'a, 'c>(
    stmts_iter: impl IntoIterator<Item = Statement<'a>> + 'c,
    has_super_class: bool,
    scope_id: ScopeId,
    ctx: &mut TraverseCtx<'a>,
) -> ClassElement<'a> {
    // Add `super(..._args);` statement and `..._args` param if class has a super class.
    // `constructor(..._args) { super(..._args); /* prop initialization */ }`
    // TODO(improve-on-babel): We can use `arguments` instead of creating `_args`.
    let mut params_rest = None;
    let stmts = if has_super_class {
        let args_binding = ctx.generate_uid("args", scope_id, SymbolFlags::FunctionScopedVariable);
        let binding_pattern = args_binding.create_binding_pattern(ctx);
        let rest_element = ctx.ast.binding_rest_element(SPAN, binding_pattern);
        params_rest = Some(ctx.ast.alloc_formal_parameter_rest(SPAN, rest_element, NONE));
        let super_call_expr = create_super_call(&args_binding, ctx);
        let super_call_stmt = ctx.ast.statement_expression(SPAN, super_call_expr);
        ctx.ast.vec_from_iter(iter::once(super_call_stmt).chain(stmts_iter))
    } else {
        ctx.ast.vec_from_iter(stmts_iter)
    };

    let items = ctx.ast.vec();
    let params = ctx.ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        items,
        params_rest,
    );

    create_class_constructor_with_params(stmts, params, scope_id, ctx)
}

//  `constructor(params) { statements }`
pub fn create_class_constructor_with_params<'a>(
    stmts: ArenaVec<'a, Statement<'a>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    scope_id: ScopeId,
    ctx: &mut TraverseCtx<'a>,
) -> ClassElement<'a> {
    let decorators = ctx.ast.vec();
    let key = PropertyKey::StaticIdentifier(
        ctx.ast.alloc_identifier_name(SPAN, Atom::from("constructor")),
    );
    let directives = ctx.ast.vec();
    let body = ctx.ast.alloc_function_body(SPAN, directives, stmts);
    let value = ctx.ast.alloc_function_with_scope_id(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(body),
        scope_id,
    );
    ClassElement::MethodDefinition(ctx.ast.alloc_method_definition(
        SPAN,
        MethodDefinitionType::MethodDefinition,
        decorators,
        key,
        value,
        MethodDefinitionKind::Constructor,
        false,
        false,
        false,
        false,
        None,
    ))
}

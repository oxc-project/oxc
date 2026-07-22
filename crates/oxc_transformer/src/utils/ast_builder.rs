use std::iter;

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::NONE};
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{GetSpan, SPAN};
use oxc_str::{Ident, static_ident};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

/// `object` -> `object.call`.
pub fn create_member_callee<'a>(
    object: Expression<'a>,
    property: Ident<'a>,
    span: Span,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let property = IdentifierName::new(SPAN, property, ctx);
    Expression::new_static_member_expression(span, object, property, false, ctx)
}

/// `object` -> `object.bind(this)`.
pub fn create_bind_call<'a>(
    callee: Expression<'a>,
    this: Expression<'a>,
    span: Span,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = create_member_callee(callee, static_ident!("bind"), span, ctx);
    let this = Argument::from(this);
    Expression::new_call_expression(span, callee, NONE, [this], false, ctx)
}

/// `object` -> `object.call(...arguments)`.
pub fn create_call_call<'a>(
    callee: Expression<'a>,
    this: Expression<'a>,
    span: Span,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = create_member_callee(callee, static_ident!("call"), span, ctx);
    let this = Argument::from(this);
    Expression::new_call_expression(span, callee, NONE, [this], false, ctx)
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
    let stmts =
        ArenaVec::from_value_in(Statement::new_return_statement(SPAN, Some(expr), ctx), ctx);
    wrap_statements_in_arrow_function_iife(stmts, scope_id, span, ctx)
}

/// Wrap statements in an IIFE (immediately invoked function expression).
///
/// `x; y; z;` -> `(() => { x; y; z; })()`
pub fn wrap_statements_in_arrow_function_iife<'a>(
    stmts: ArenaVec<'a, Statement<'a>>,
    scope_id: ScopeId,
    span: Span,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let kind = FormalParameterKind::ArrowFormalParameters;
    let params = FormalParameters::boxed(SPAN, kind, [], NONE, ctx);
    let body = FunctionBody::boxed(SPAN, [], stmts, ctx);
    let arrow = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
        SPAN, false, false, NONE, params, NONE, body, scope_id, false, false, ctx,
    );
    Expression::new_call_expression(span, arrow, NONE, [], false, ctx)
}

/// `object` -> `object.prototype`.
pub fn create_prototype_member<'a>(
    object: Expression<'a>,
    span: Span,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let property = IdentifierName::new(SPAN, "prototype", ctx);
    let static_member =
        MemberExpression::new_static_member_expression(span, object, property, false, ctx);
    Expression::from(static_member)
}

/// `object` -> `object.a`.
pub fn create_property_access<'a>(
    span: Span,
    object: Expression<'a>,
    property: &str,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let property = IdentifierName::new(SPAN, Str::from_str_in(property, ctx), ctx);
    Expression::new_static_member_expression(span, object, property, false, ctx)
}

/// `this.property`
#[inline]
pub fn create_this_property_access<'a>(
    span: Span,
    property: Ident<'a>,
    ctx: &TraverseCtx<'a>,
) -> MemberExpression<'a> {
    let object = Expression::new_this_expression(span, ctx);
    let property = IdentifierName::new(SPAN, property, ctx);
    MemberExpression::new_static_member_expression(span, object, property, false, ctx)
}

/// `this.property`
#[inline]
pub fn create_this_property_assignment<'a>(
    span: Span,
    property: Ident<'a>,
    ctx: &TraverseCtx<'a>,
) -> AssignmentTarget<'a> {
    AssignmentTarget::from(create_this_property_access(span, property, ctx))
}

/// Create assignment to a binding.
pub fn create_assignment<'a>(
    binding: &BoundIdentifier<'a>,
    value: Expression<'a>,
    span: Span,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    Expression::new_assignment_expression(
        span,
        AssignmentOperator::Assign,
        binding.create_target(ReferenceFlags::Write, ctx),
        value,
        ctx,
    )
}

/// `super(...args);`
pub fn create_super_call<'a>(
    args_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    Expression::new_call_expression(
        SPAN,
        Expression::new_super(SPAN, ctx),
        NONE,
        [Argument::new_spread_element(SPAN, args_binding.create_read_expression(ctx), ctx)],
        false,
        ctx,
    )
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
        let rest_element =
            BindingRestElement::new(SPAN, args_binding.create_binding_pattern(ctx), ctx);
        params_rest = Some(FormalParameterRest::boxed(SPAN, [], rest_element, NONE, ctx));
        ArenaVec::from_iter_in(
            iter::once(Statement::new_expression_statement(
                SPAN,
                create_super_call(&args_binding, ctx),
                ctx,
            ))
            .chain(stmts_iter),
            ctx,
        )
    } else {
        ArenaVec::from_iter_in(stmts_iter, ctx)
    };

    let params =
        FormalParameters::boxed(SPAN, FormalParameterKind::FormalParameter, [], params_rest, ctx);

    create_class_constructor_with_params(stmts, params, scope_id, ctx)
}

//  `constructor(params) { statements }`
pub fn create_class_constructor_with_params<'a>(
    stmts: ArenaVec<'a, Statement<'a>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    scope_id: ScopeId,
    ctx: &TraverseCtx<'a>,
) -> ClassElement<'a> {
    create_class_method(
        ArenaVec::new_in(ctx),
        PropertyKey::new_static_identifier(SPAN, "constructor", ctx),
        MethodDefinitionKind::Constructor,
        params,
        None,
        stmts,
        false,
        false,
        scope_id,
        ctx,
    )
}

/// Create a `MethodDefinition` class element wrapping a function expression.
pub fn create_class_method<'a>(
    decorators: ArenaVec<'a, Decorator<'a>>,
    key: PropertyKey<'a>,
    kind: MethodDefinitionKind,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    stmts: ArenaVec<'a, Statement<'a>>,
    computed: bool,
    is_static: bool,
    scope_id: ScopeId,
    ctx: &TraverseCtx<'a>,
) -> ClassElement<'a> {
    ClassElement::new_method_definition(
        SPAN,
        MethodDefinitionType::MethodDefinition,
        decorators,
        key,
        Function::boxed_with_scope_id(
            SPAN,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            return_type,
            Some(FunctionBody::boxed(SPAN, [], stmts, ctx)),
            scope_id,
            ctx,
        ),
        kind,
        computed,
        is_static,
        false,
        false,
        None,
        ctx,
    )
}

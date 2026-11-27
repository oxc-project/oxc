use std::iter;

use oxc_allocator::{ArenaBox, ArenaVec, ReplaceWith, TakeIn};
use oxc_ast::ast::*;
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{GetSpan, SPAN, Span};
use oxc_str::{Ident, static_ident};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

/// Convert a concise arrow body to a block body with an explicit return, and return the block.
pub fn arrow_function_body_as_function_body_mut<'a, 'b>(
    body: &'b mut ArrowFunctionBody<'a>,
    ctx: &TraverseCtx<'a>,
) -> &'b mut FunctionBody<'a> {
    if body.is_expression() {
        body.replace_with(|body| {
            let expression = body.into_expression();
            let span = expression.span();
            let statement = Statement::new_return_statement(span, Some(expression), ctx);
            ArrowFunctionBody::FunctionBody(FunctionBody::boxed(span, [], [statement], ctx))
        });
    }
    body.as_function_body_mut().unwrap()
}

/// Take a non-computed string, numeric or BigInt literal property key as an expression.
///
/// `'x'` -> `"x"`, `123` -> `123`, `1n` -> `1n`.
/// Returns `None` for other keys.
pub fn take_literal_property_key<'a>(
    key: &mut PropertyKey<'a>,
    ctx: &TraverseCtx<'a>,
) -> Option<Expression<'a>> {
    if !matches!(
        key,
        PropertyKey::StaticStringLiteral(_)
            | PropertyKey::StaticNumericLiteral(_)
            | PropertyKey::StaticBigIntLiteral(_)
    ) {
        return None;
    }
    let expr = match key.take_in(ctx) {
        PropertyKey::StaticStringLiteral(lit) => Expression::StringLiteral(lit),
        PropertyKey::StaticNumericLiteral(lit) => Expression::NumericLiteral(lit),
        PropertyKey::StaticBigIntLiteral(lit) => Expression::BigIntLiteral(lit),
        _ => unreachable!(),
    };
    Some(expr)
}

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
    Expression::new_call_expression(span, callee, None, [this], false, ctx)
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
    Expression::new_call_expression(span, callee, None, [this], false, ctx)
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
    let params = FormalParameters::boxed(SPAN, kind, [], None, ctx);
    let body = FunctionBody::boxed(SPAN, [], stmts, ctx);
    let arrow = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
        SPAN,
        false,
        None,
        params,
        None,
        ArrowFunctionBody::FunctionBody(body),
        scope_id,
        false,
        false,
        ctx,
    );
    Expression::new_call_expression(span, arrow, None, [], false, ctx)
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
        None,
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
        params_rest = Some(FormalParameterRest::boxed(SPAN, [], rest_element, None, ctx));
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
        SPAN,
        ArenaVec::new_in(ctx),
        PropertyKey::new_static_identifier(SPAN, "constructor", ctx),
        MethodDefinitionKind::Constructor,
        params,
        None,
        stmts,
        false,
        scope_id,
        ctx,
    )
}

/// Create a `MethodDefinition` class element wrapping a function expression.
///
/// `span` is used for the method and its body.
pub fn create_class_method<'a>(
    span: Span,
    decorators: ArenaVec<'a, Decorator<'a>>,
    key: PropertyKey<'a>,
    kind: MethodDefinitionKind,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    stmts: ArenaVec<'a, Statement<'a>>,
    is_static: bool,
    scope_id: ScopeId,
    ctx: &TraverseCtx<'a>,
) -> ClassElement<'a> {
    ClassElement::new_method_definition(
        span,
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
            None,
            None,
            params,
            return_type,
            Some(FunctionBody::boxed(span, [], stmts, ctx)),
            scope_id,
            ctx,
        ),
        kind,
        is_static,
        false,
        false,
        None,
        ctx,
    )
}

/// Create a getter or setter method that accesses a private backing field.
///
/// Instance: `get <key>() { return this.#<storage_name>; }`
/// Static:   `get <key>() { return ClassName.#<storage_name>; }`
///
/// The method and its body get `span`, and the statement in the body gets `statement_span`, so
/// stepping into the method maps to the source of the auto-accessor it is generated from.
/// The two must differ: a source map records no mapping for a position equal to the last one,
/// which would leave the statement's line unmapped.
pub fn create_accessor_method<'a>(
    span: Span,
    statement_span: Span,
    decorators: ArenaVec<'a, Decorator<'a>>,
    key: PropertyKey<'a>,
    kind: MethodDefinitionKind,
    is_static: bool,
    storage_name: Str<'a>,
    object_binding: Option<&BoundIdentifier<'a>>,
    class_scope_id: ScopeId,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    ctx: &mut TraverseCtx<'a>,
) -> ClassElement<'a> {
    let is_getter = kind == MethodDefinitionKind::Get;
    let scope_flags = ScopeFlags::Function
        | ScopeFlags::StrictMode
        | if is_getter { ScopeFlags::GetAccessor } else { ScopeFlags::SetAccessor };
    let scope_id = ctx.create_child_scope(class_scope_id, scope_flags);

    // For static accessors use class name, for instance accessors use `this`.
    let create_object = |ctx: &mut TraverseCtx<'a>| -> Expression<'a> {
        if let Some(binding) = object_binding {
            binding.create_read_expression(ctx)
        } else {
            Expression::new_this_expression(SPAN, ctx)
        }
    };

    let (params, body_stmt) = if is_getter {
        let params =
            FormalParameters::boxed(SPAN, FormalParameterKind::FormalParameter, [], None, ctx);
        let field_expr = Expression::new_private_field_expression(
            SPAN,
            create_object(ctx),
            PrivateIdentifier::new(SPAN, storage_name, ctx),
            false,
            ctx,
        );
        let stmt = Statement::new_return_statement(statement_span, Some(field_expr), ctx);
        (params, stmt)
    } else {
        // The body reads `object_binding`, so a parameter with the same name would shadow it,
        // as in `class value { static accessor x; }`. Use a unique name in that case.
        let flags = SymbolFlags::FunctionScopedVariable;
        let value_binding = if object_binding.is_some_and(|binding| binding.name == "value") {
            ctx.generate_uid("value", scope_id, flags)
        } else {
            ctx.generate_binding(static_ident!("value"), scope_id, flags)
        };
        let param =
            FormalParameter::new_plain(SPAN, value_binding.create_binding_pattern(ctx), ctx);
        let params =
            FormalParameters::boxed(SPAN, FormalParameterKind::FormalParameter, [param], None, ctx);
        let assign = Expression::new_assignment_expression(
            SPAN,
            AssignmentOperator::Assign,
            AssignmentTarget::new_private_field_expression(
                SPAN,
                create_object(ctx),
                PrivateIdentifier::new(SPAN, storage_name, ctx),
                false,
                ctx,
            ),
            value_binding.create_read_expression(ctx),
            ctx,
        );
        let stmt = Statement::new_expression_statement(statement_span, assign, ctx);
        (params, stmt)
    };

    create_class_method(
        span,
        decorators,
        key,
        kind,
        params,
        return_type,
        ArenaVec::from_value_in(body_stmt, ctx),
        is_static,
        scope_id,
        ctx,
    )
}

/// `export default Class`
pub fn create_export_default_class_reference<'a>(
    class_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    let export_default_class_reference = ModuleDeclaration::new_export_default_declaration(
        SPAN,
        ExportDefaultDeclarationKind::Identifier(ArenaBox::new_in(
            class_binding.create_read_reference(ctx),
            ctx,
        )),
        ctx,
    );
    Statement::from(export_default_class_reference)
}

/// `export { Class }`
pub fn create_export_named_class_reference<'a>(
    class_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    let kind = ImportOrExportKind::Value;
    let local = ModuleExportName::IdentifierReference(class_binding.create_read_reference(ctx));
    let exported = ModuleExportName::new_identifier_name(SPAN, class_binding.name, ctx);
    let specifiers = [ExportSpecifier::new(SPAN, local, exported, kind, ctx)];
    let export_class_reference =
        ModuleDeclaration::new_export_named_declaration(SPAN, specifiers, kind, ctx);
    Statement::from(export_class_reference)
}

/// Create `var` declaration.
pub fn create_variable_declaration<'a>(
    binding: &BoundIdentifier<'a>,
    init: Expression<'a>,
    ctx: &TraverseCtx<'a>,
) -> Statement<'a> {
    let kind = VariableDeclarationKind::Var;
    let declarator = VariableDeclarator::new(
        SPAN,
        binding.create_binding_pattern(ctx),
        None,
        Some(init),
        false,
        ctx,
    );
    Statement::new_variable_declaration(SPAN, kind, [declarator], false, ctx)
}

/// Create `new WeakSet()` expression.
pub fn create_new_weakset<'a>(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
    let weak_set = static_ident!("WeakSet");
    let symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), weak_set);
    let ident = ctx.create_ident_expr(SPAN, weak_set, symbol_id, ReferenceFlags::Read);
    Expression::new_new_expression_with_pure(SPAN, ident, None, [], true, ctx)
}
